use std::time::Duration;

use kinesin::config::ModelConfig;
use kinesin::core::{self, ModelReply};
use kinesin::model::{
    ModelClient, ModelOptions, StreamDecoder, TextObserver, decode_reply, prepare,
};

/// The protocol tests assert on the decoded reply; usage is exercised separately.
fn decoded(bytes: &[u8]) -> Result<kinesin::core::ModelReply, String> {
    decode_reply(bytes).map(|outcome| outcome.reply)
}
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn response(content: Value, calls: Value, reason: &str) -> Vec<u8> {
    let mut message = json!({"role":"assistant","content":content});
    if !calls.is_null() {
        message["tool_calls"] = calls;
    }
    serde_json::to_vec(&json!({"choices":[{"index":0,"finish_reason":reason,"message":message}]}))
        .unwrap()
}

#[test]
fn full_reply_validation_never_promotes_partial_calls() {
    let call = json!({"id":"call-1","type":"function","function":{
        "name":"read_file","arguments":"{\"path\":\"project.txt\"}"
    }});
    let tools = json!([call.clone()]);
    let valid = decoded(&response(Value::Null, tools.clone(), "tool_calls")).unwrap();
    assert!(matches!(valid, ModelReply::ToolCalls {calls,..} if calls.len()==1));
    assert!(matches!(
        decoded(&response(Value::Null, tools.clone(), "length")).unwrap(),
        ModelReply::Incomplete(_)
    ));
    assert!(decoded(&response(json!("done"), tools, "stop")).is_err());
    assert!(
        decoded(&response(
            Value::Null,
            json!([call.clone(), call]),
            "tool_calls"
        ))
        .is_err()
    );
    assert!(decoded(&response(json!(""), Value::Null, "stop")).is_err());
    assert!(decoded(br#"{"choices":[],"choices":[]}"#).is_err());
    assert!(decoded(&response(json!("Answer"), Value::Null, "mystery")).is_err());
    let mut contradictory: Value =
        serde_json::from_slice(&response(json!("Answer"), Value::Null, "stop")).unwrap();
    contradictory["error"] = json!({"message":"failed"});
    assert_eq!(
        decoded(&serde_json::to_vec(&contradictory).unwrap()).unwrap_err(),
        "provider_error"
    );
    assert_eq!(
        decoded(&response(
            json!("Use read_file(project.txt)"),
            Value::Null,
            "stop"
        ))
        .unwrap(),
        ModelReply::Answer("Use read_file(project.txt)".into())
    );
}

fn profile(origin: String) -> ModelConfig {
    ModelConfig {
        id: "fixture".into(),
        base_url: origin,
        model_id: "fixture".into(),
        context_size: 4096,
        verified_slots: 1,
        temperature: 0.0,
        stream: false,
        request_timeout_s: 2,
        connect_timeout_s: 1,
        read_timeout_s: 1,
        model_queue_timeout_s: 1,
    }
}

fn request(origin: &str) -> kinesin::model::PreparedRequest {
    request_options(origin, false, 1048576)
}

fn request_options(origin: &str, stream: bool, limit: usize) -> kinesin::model::PreparedRequest {
    let (state, _) = core::initiate("System".into(), "Question".into()).unwrap();
    prepare(
        state.messages(),
        &ModelOptions {
            origin: origin.into(),
            served_model: "fixture".into(),
            temperature: 0.0,
            max_output_tokens: 64,
            max_request_bytes: 131072,
            max_response_bytes: limit,
            stream,
            tools: Vec::new(),
            constraint: None,
        },
    )
    .unwrap()
}

/// This fixture accepts one bounded Content-Length request from reqwest. It is
/// deliberately not a general HTTP server or the product's HTTP implementation.
async fn fixture(reply: Vec<u8>) -> (String, tokio::task::JoinHandle<Vec<u8>>) {
    fixture_parts(vec![reply], None).await
}

async fn fixture_parts(
    reply: Vec<Vec<u8>>,
    mut gate: Option<tokio::sync::oneshot::Receiver<()>>,
) -> (String, tokio::task::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(5), async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            let (header_end, body_len) = loop {
                let mut chunk = [0_u8; 1024];
                let count = stream.read(&mut chunk).await.unwrap();
                assert!(count > 0 && bytes.len() + count <= 16384);
                bytes.extend_from_slice(&chunk[..count]);
                if let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                    let headers = std::str::from_utf8(&bytes[..end]).unwrap();
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().unwrap())
                        })
                        .unwrap();
                    assert!(length <= 8192);
                    break (end + 4, length);
                }
            };
            while bytes.len() < header_end + body_len {
                let mut chunk = [0_u8; 1024];
                let count = stream.read(&mut chunk).await.unwrap();
                assert!(count > 0 && bytes.len() + count <= 16384);
                bytes.extend_from_slice(&chunk[..count]);
            }
            let body = bytes[header_end..header_end + body_len].to_vec();
            for part in reply {
                if stream.write_all(&part).await.is_err() {
                    break;
                }
                if let Some(wait) = gate.take() {
                    wait.await.unwrap();
                }
            }
            body
        })
        .await
        .expect("bounded fixture lifetime")
    });
    (origin, handle)
}

#[tokio::test]
async fn http_sends_exact_prepared_bytes_and_normalizes_the_reply() {
    let body = response(json!("observed"), Value::Null, "stop");
    let mut reply=format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).into_bytes();
    reply.extend_from_slice(&body);
    let (origin, server) = fixture(reply).await;
    let prepared = request(&origin);
    let client = ModelClient::http(&profile(origin), 1048576).unwrap();
    assert_eq!(
        client.send(&prepared).await.unwrap().reply,
        ModelReply::Answer("observed".into())
    );
    assert_eq!(server.await.unwrap(), prepared.bytes());
}

#[tokio::test]
async fn redirect_does_not_contact_the_supplied_destination() {
    let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let reply=format!("HTTP/1.1 302 Found\r\nLocation: http://{}/forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",target.local_addr().unwrap()).into_bytes();
    let (origin, server) = fixture(reply).await;
    let client = ModelClient::http(&profile(origin.clone()), 1048576).unwrap();
    assert_eq!(
        client.send(&request(&origin)).await.unwrap_err(),
        "http_status_302"
    );
    server.await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(40), target.accept())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn chunked_error_body_cannot_bypass_the_size_limit() {
    let reply=b"HTTP/1.1 503 Busy\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n10\r\n0123456789abcdef\r\n10\r\n0123456789abcdef\r\n0\r\n\r\n".to_vec();
    let (origin, server) = fixture(reply).await;
    let client = ModelClient::http(&profile(origin.clone()), 20).unwrap();
    assert_eq!(
        client.send(&request(&origin)).await.unwrap_err(),
        "response_bytes_limit"
    );
    server.await.unwrap();
}

#[tokio::test]
async fn unexpected_encoding_and_destination_are_rejected() {
    let reply=b"HTTP/1.1 200 OK\r\nContent-Encoding: mystery\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec();
    let (origin, server) = fixture(reply).await;
    let client = ModelClient::http(&profile(origin.clone()), 1048576).unwrap();
    assert_eq!(
        client
            .send(&request("http://127.0.0.1:1"))
            .await
            .unwrap_err(),
        "prepared_destination_mismatch"
    );
    assert_eq!(
        client.send(&request(&origin)).await.unwrap_err(),
        "unsupported_content_encoding"
    );
    server.await.unwrap();
}

fn sse(value: Value) -> String {
    format!("data: {value}\n\n")
}

fn delta(value: Value, finish: Option<&str>) -> String {
    sse(json!({"choices":[{"index":0,"delta":value,"finish_reason":finish}]}))
}

fn role() -> String {
    delta(json!({"role":"assistant","content":null}), None)
}

fn end(reason: &str) -> String {
    format!("{}data: [DONE]\n\n", delta(json!({}), Some(reason)))
}

fn text_stream(text: &str) -> String {
    format!(
        "{}{}{}",
        role(),
        delta(json!({"content":text}), None),
        end("stop")
    )
}

fn parse_stream(bytes: &[u8]) -> Result<ModelReply, String> {
    let mut decoder = StreamDecoder::new(1048576)?;
    decoder.push(bytes)?;
    decoder.finish().map(|outcome| outcome.reply)
}

fn tool(index: usize, id: &str, arguments: &str) -> Value {
    json!({"index":index,"id":id,"type":"function","function":{
        "name":"read_file","arguments":arguments
    }})
}

fn http_response(status: &str, mime: &str, body: &[u8]) -> Vec<u8> {
    let mut reply = format!("HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).into_bytes();
    reply.extend_from_slice(body);
    reply
}

#[tokio::test]
async fn prepared_response_limit_is_enforced_for_success_error_and_chunked_bodies() {
    let body = response(
        json!("larger than the run's response budget"),
        Value::Null,
        "stop",
    );
    for reply in [
        http_response("200 OK", "application/json", &body),
        http_response("503 Busy", "application/json", &body),
        b"HTTP/1.1 503 Busy\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n10\r\n0123456789abcdef\r\n10\r\n0123456789abcdef\r\n0\r\n\r\n".to_vec(),
    ] {
        let (origin, server) = fixture(reply).await;
        let prepared = request_options(&origin, false, 20);
        assert_eq!(prepared.max_response_bytes(), 20);
        let client = ModelClient::http(&profile(origin), 1048576).unwrap();
        assert_eq!(client.send(&prepared).await.unwrap_err(), "response_bytes_limit");
        server.await.unwrap();
    }
    let (origin, server) = fixture(http_response("200 OK", "application/json", &body)).await;
    let client = ModelClient::http(&profile(origin.clone()), body.len()).unwrap();
    assert!(
        client
            .send(&request_options(&origin, false, body.len()))
            .await
            .is_ok()
    );
    server.await.unwrap();
}

#[test]
fn actual_pinned_text_and_tool_streams_accept_every_byte_fragment() {
    for (bytes, expected) in [
        (
            &include_bytes!("fixtures/live/text-stream.response.sse")[..],
            ModelReply::Answer("Hello Kinesin.".into()),
        ),
        (
            &include_bytes!("fixtures/live/tool-call-stream.response.sse")[..],
            ModelReply::ToolCalls {
                content: None,
                calls: vec![core::ToolCall {
                    id: "SzVKUzX1kFAFNhAPi1QPHdy5jz29jxCD".into(),
                    name: "read_file".into(),
                    arguments: "{\"path\":\"project.txt\"}".into(),
                }],
            },
        ),
    ] {
        let mut decoder = StreamDecoder::new(bytes.len()).unwrap();
        for byte in bytes {
            decoder.push(std::slice::from_ref(byte)).unwrap();
        }
        assert_eq!(decoder.finish().unwrap().reply, expected);
    }
}

#[test]
fn stream_framing_handles_unicode_multiline_data_comments_and_all_line_endings() {
    // JSON whitespace may span multiple SSE data lines. Unknown fields and
    // comments have no provider meaning, but still count toward byte bounds.
    let source = format!(
        "\u{feff}: comment\nid: ignored\nretry: 1\nunknown: x\n\nevent: message\ndata: {{\ndata: \"choices\": [{{\"index\":0,\"delta\":{{\"role\":\"assistant\"}}}}]}}\n\n{}{}{}",
        delta(json!({}), None),
        delta(json!({"content":"雪🙂"}), None),
        end("stop")
    );
    for ending in ["\n", "\r\n", "\r"] {
        let bytes = source.replace('\n', ending).into_bytes();
        for split in 0..=bytes.len() {
            let mut decoder = StreamDecoder::new(bytes.len()).unwrap();
            decoder.push(&bytes[..split]).unwrap();
            decoder.push(&bytes[split..]).unwrap();
            assert_eq!(
                decoder.finish().unwrap().reply,
                ModelReply::Answer("雪🙂".into())
            );
        }
    }
    // A BOM only has special meaning at the beginning of the stream. Later
    // BOM-prefixed field names are unknown fields, not another data event.
    let source = format!(
        "{}\u{feff}data: this is an unknown field\n\n{}",
        role(),
        text_stream("ok")
    );
    assert_eq!(
        parse_stream(source.as_bytes()).unwrap(),
        ModelReply::Answer("ok".into())
    );
}

#[test]
fn interleaved_tools_preserve_identity_index_order_and_argument_fragments() {
    let source = format!(
        "{}{}{}{}{}",
        role(),
        delta(
            json!({"content":"","tool_calls":[tool(1,"second","{\"path\":"), tool(0,"first","{")]}),
            None
        ),
        delta(
            json!({"tool_calls":[{"index":0,"function":{"arguments":"\"path\":\"a\"}"}}]}),
            None
        ),
        delta(
            json!({"tool_calls":[{"index":1,"id":"second","function":{"name":"read_file","arguments":"\"b\"}"}}]}),
            None
        ),
        end("tool_calls")
    );
    let ModelReply::ToolCalls { content, calls } = parse_stream(source.as_bytes()).unwrap() else {
        panic!("expected complete tool calls");
    };
    assert_eq!(content, Some(String::new()));
    assert_eq!(
        calls.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert_eq!(calls[0].arguments, "{\"path\":\"a\"}");
    assert_eq!(calls[1].arguments, "{\"path\":\"b\"}");
}

#[test]
fn malformed_or_contradictory_streams_never_return_a_complete_reply() {
    let valid_prefix = format!("{}{}", role(), delta(json!({"content":"visible"}), None));
    let finish = delta(json!({}), Some("stop"));
    let usage = sse(json!({"choices":[],"usage":{"total_tokens":2}}));
    let cases = [
        (
            format!(
                "{valid_prefix}{}",
                sse(
                    json!({"choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"error":{"message":"failed"}})
                )
            ),
            "provider_error",
        ),
        (valid_prefix.clone(), "stream_incomplete"),
        (format!("{valid_prefix}{finish}"), "stream_incomplete"),
        (
            format!("{valid_prefix}data: [DONE]\n\n"),
            "stream_done_before_finish",
        ),
        (
            format!("{valid_prefix}{}data: [DONE]\n\n", end("stop")),
            "stream_after_done",
        ),
        (
            format!("{valid_prefix}{finish}{finish}data: [DONE]\n\n"),
            "stream_after_finish",
        ),
        (
            format!("{valid_prefix}{usage}{}", end("stop")),
            "protocol_stream_usage",
        ),
        (
            format!("{valid_prefix}{finish}{usage}{usage}data: [DONE]\n\n"),
            "protocol_stream_usage",
        ),
        (
            format!(
                "{valid_prefix}{finish}{}data: [DONE]\n\n",
                sse(json!({"choices":[]}))
            ),
            "protocol_stream_usage",
        ),
        (
            format!(
                "{valid_prefix}{finish}{}data: [DONE]\n\n",
                delta(json!({"content":"late"}), None)
            ),
            "stream_after_finish",
        ),
        (
            format!("{valid_prefix}{}", end("mystery")),
            "protocol_finish_reason",
        ),
        (
            format!(
                "{}{}",
                delta(json!({"role":"user","content":"x"}), None),
                end("stop")
            ),
            "protocol_choice_or_role",
        ),
        (
            format!("{}{}", delta(json!({"content":"x"}), None), end("stop")),
            "protocol_stream_missing_role",
        ),
        (
            format!(
                "{}{}",
                role(),
                sse(json!({"choices":[{"index":1,"delta":{}}]}))
            ),
            "protocol_choice_or_role",
        ),
        (
            format!(
                "{}{}",
                role(),
                sse(json!({"choices":[{"index":0,"delta":{}},{"index":0,"delta":{}}]}))
            ),
            "protocol_choice_count",
        ),
        (
            format!("{}{}", role(), sse(json!({"choices":[{"index":0}]}))),
            "protocol_stream_delta",
        ),
        (
            format!("{valid_prefix}data: {{\"choices\":[],\"choices\":[]}}\n\n"),
            "invalid_stream_json",
        ),
        (
            format!("{valid_prefix}event: error\ndata: {{\"choices\":[]}}\n\n"),
            "unsupported_stream_event",
        ),
        (
            format!("{}data: [DONE]", format_args!("{valid_prefix}{finish}")),
            "stream_incomplete",
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(
            parse_stream(source.as_bytes()).unwrap_err(),
            expected,
            "{source}"
        );
    }
    let mut invalid_utf8 = role().into_bytes();
    invalid_utf8.extend_from_slice(b"data: \xff\n\n");
    assert_eq!(
        parse_stream(&invalid_utf8).unwrap_err(),
        "stream_invalid_utf8"
    );
    let mut decoder = StreamDecoder::new(1048576).unwrap();
    assert!(decoder.push(b"data: invalid\n\n").is_err());
    assert!(
        decoder
            .push(text_stream("later valid input").as_bytes())
            .is_err()
    );
    assert!(decoder.finish().is_err());
}

#[test]
fn partial_tool_calls_and_invalid_identities_are_never_executable() {
    let cases = [
        json!([{"index":0,"function":{"arguments":"{}"}}]),
        json!([tool(1, "gap", "{}")]),
        json!([tool(0, "duplicate", "{}"), tool(1, "duplicate", "{}")]),
        json!([tool(0, "same-index", "{}"), tool(0, "same-index", "{}")]),
        json!([tool(24, "over-count", "{}")]),
        json!([tool(0, "\ninvalid", "{}")]),
        json!([{"index":0,"id":"id","type":"unexpected","function":{"name":"read_file","arguments":"{}"}}]),
    ];
    for calls in cases {
        let source = format!(
            "{}{}{}",
            role(),
            delta(json!({"tool_calls":calls}), None),
            end("tool_calls")
        );
        assert!(parse_stream(source.as_bytes()).is_err(), "{source}");
    }
    for changed in [
        json!({"index":0,"id":"different"}),
        json!({"index":0,"function":{"name":"list_files"}}),
    ] {
        let source = format!(
            "{}{}{}{}",
            role(),
            delta(json!({"tool_calls":[tool(0,"original","{}")]}), None),
            delta(json!({"tool_calls":[changed]}), None),
            end("tool_calls")
        );
        assert_eq!(
            parse_stream(source.as_bytes()).unwrap_err(),
            "protocol_stream_identity"
        );
    }
    let partial = format!(
        "{}{}",
        role(),
        delta(json!({"tool_calls":[tool(0,"id","{\"path\":")]}), None)
    );
    assert_eq!(
        parse_stream(partial.as_bytes()).unwrap_err(),
        "stream_incomplete"
    );
    assert_eq!(
        parse_stream(format!("{partial}{}", end("length")).as_bytes()).unwrap(),
        ModelReply::Incomplete("generation_length".into())
    );
    assert!(parse_stream(format!("{partial}{}", end("stop")).as_bytes()).is_err());
}

#[test]
fn frames_total_bytes_and_combined_tool_arguments_have_independent_bounds() {
    for source in [
        format!("data: {}", "x".repeat(65536)),
        ": ignored\n".repeat(7000),
    ] {
        let mut decoder = StreamDecoder::new(1048576).unwrap();
        assert_eq!(
            decoder.push(source.as_bytes()).unwrap_err(),
            "stream_frame_bytes_limit"
        );
    }
    let source = format!("{}{}", text_stream("ok"), ": ignored\n\n".repeat(50));
    let mut decoder = StreamDecoder::new(source.len() - 1).unwrap();
    for byte in source.as_bytes().split_last().unwrap().1 {
        decoder.push(std::slice::from_ref(byte)).unwrap();
    }
    assert_eq!(
        decoder
            .push(&source.as_bytes()[source.len() - 1..])
            .unwrap_err(),
        "response_bytes_limit"
    );
    let source = format!(
        "{}{}{}",
        role(),
        delta(
            json!({"reasoning_content":"x".repeat(2000),"content":"ok"}),
            None
        ),
        end("stop")
    );
    let mut decoder = StreamDecoder::new(1000).unwrap();
    assert_eq!(
        decoder.push(source.as_bytes()).unwrap_err(),
        "response_bytes_limit"
    );
    let source = format!(
        "{}{}{}",
        role(),
        delta(
            json!({"tool_calls":[tool(0,"first",&"x".repeat(40000))]}),
            None
        ),
        delta(
            json!({"tool_calls":[tool(1,"second",&"y".repeat(30000))]}),
            None
        )
    );
    assert_eq!(
        parse_stream(source.as_bytes()).unwrap_err(),
        "stream_argument_bytes_limit"
    );
}

#[tokio::test]
async fn streaming_http_requires_correct_mime_completion_and_run_response_cap() {
    let good = text_stream("雪🙂");
    for (mime, body, cap, expected) in [
        (
            "application/json",
            good.clone(),
            1048576,
            "unsupported_stream_content_type",
        ),
        (
            "text/event-stream",
            format!("{}{}", role(), delta(json!({"content":"partial"}), None)),
            1048576,
            "stream_incomplete",
        ),
        (
            "text/event-stream",
            good.clone(),
            20,
            "response_bytes_limit",
        ),
    ] {
        let (origin, server) = fixture(http_response("200 OK", mime, body.as_bytes())).await;
        let client = ModelClient::http(&profile(origin.clone()), 1048576).unwrap();
        assert_eq!(
            client
                .send(&request_options(&origin, true, cap))
                .await
                .unwrap_err(),
            expected
        );
        server.await.unwrap();
    }
    // Transfer encoding and arbitrary body chunking are independent from SSE.
    let mut reply=b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream; charset=utf-8\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".to_vec();
    for byte in good.bytes() {
        reply.extend_from_slice(&[b'1', b'\r', b'\n', byte, b'\r', b'\n']);
    }
    reply.extend_from_slice(b"0\r\n\r\n");
    let (origin, server) = fixture(reply).await;
    let prepared = request_options(&origin, true, good.len());
    assert!(prepared.is_streaming());
    let client = ModelClient::http(&profile(origin), 1048576).unwrap();
    assert_eq!(
        client.send(&prepared).await.unwrap().reply,
        ModelReply::Answer("雪🙂".into())
    );
    server.await.unwrap();
}

#[tokio::test]
async fn text_is_visible_before_completion_but_tool_arguments_and_private_fields_are_not() {
    let initial = format!(
        "{}{}",
        role(),
        delta(
            json!({"content":"provisional","reasoning_content":"private reasoning"}),
            None
        )
    );
    let final_part = format!(
        "{}{}",
        delta(
            json!({"tool_calls":[tool(0,"id","{\"path\":\"private.txt\"}")]}),
            None
        ),
        end("tool_calls")
    );
    let mut first=format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",initial.len()+final_part.len()).into_bytes();
    first.extend_from_slice(initial.as_bytes());
    let (release, gate) = tokio::sync::oneshot::channel();
    let (origin, server) = fixture_parts(vec![first, final_part.into_bytes()], Some(gate)).await;
    let client = ModelClient::http(&profile(origin.clone()), 1048576).unwrap();
    let prepared = request_options(&origin, true, 1048576);
    let (mut observer, mut receiver) = TextObserver::bounded();
    let send = tokio::spawn(async move { client.send_with_text(&prepared, &mut observer).await });
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(1), receiver.recv())
            .await
            .unwrap()
            .unwrap(),
        "provisional"
    );
    assert!(!send.is_finished());
    release.send(()).unwrap();
    assert!(
        matches!(send.await.unwrap().unwrap().reply,ModelReply::ToolCalls{calls,..} if calls.len()==1)
    );
    assert!(receiver.recv().await.is_none());
    assert!(!receiver.is_lagged());
    server.await.unwrap();
}

#[tokio::test]
async fn display_disconnects_on_lag_and_cannot_block_or_cancel_model_completion() {
    let text = "雪".repeat(100000);
    let client = ModelClient::scripted([ModelReply::Answer(text.clone()).into()]);
    let (mut observer, mut receiver) = TextObserver::bounded();
    let result = client
        .send_with_text(&request_options("scripted", true, 1048576), &mut observer)
        .await
        .unwrap()
        .reply;
    assert_eq!(result, ModelReply::Answer(text));
    assert!(receiver.is_lagged());
    let mut messages = 0;
    let mut bytes = 0;
    while let Some(frame) = receiver.recv().await {
        assert!(!frame.is_empty() && frame.len() <= 2048);
        assert!(frame.chars().all(|ch| ch == '雪'));
        messages += 1;
        bytes += frame.len();
    }
    assert_eq!(messages, 128);
    assert!(bytes <= 256 * 1024);
    let (mut observer, receiver) = TextObserver::bounded();
    drop(receiver);
    let client = ModelClient::scripted([ModelReply::Answer("still completed".into()).into()]);
    assert_eq!(
        client
            .send_with_text(&request_options("scripted", true, 1048576), &mut observer)
            .await
            .unwrap()
            .reply,
        ModelReply::Answer("still completed".into())
    );
}

#[tokio::test]
async fn scripted_normalized_payload_still_obeys_the_per_run_cap() {
    let client = ModelClient::scripted([ModelReply::Answer("\\".repeat(20)).into()]);
    assert_eq!(
        client
            .send(&request_options("scripted", false, 30))
            .await
            .unwrap_err(),
        "response_bytes_limit"
    );
}
