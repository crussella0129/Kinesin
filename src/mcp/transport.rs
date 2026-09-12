//! Apply byte and frame-count limits before the SDK buffers or decodes a line.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, ReadBuf};

pub(super) struct BoundedReader<R> {
    inner: R,
    line_bytes: usize,
    total_bytes: usize,
    line_limit: usize,
    total_limit: usize,
    frames: usize,
    frame_limit: usize,
    failed: bool,
}

impl<R> BoundedReader<R> {
    pub(super) fn new(inner: R, line_limit: usize, total_limit: usize, frame_limit: usize) -> Self {
        Self {
            inner,
            line_bytes: 0,
            total_bytes: 0,
            line_limit,
            total_limit,
            frames: 0,
            frame_limit,
            failed: false,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for BoundedReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        dest: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.failed {
            return Poll::Ready(Err(io::Error::other("MCP wire limit")));
        }
        if dest.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        // Small fixed storage prevents the downstream decoder's requested read
        // size from controlling an allocation. Counters survive cancelled polls.
        let mut chunk = [0_u8; 8192];
        let length = dest.remaining().min(chunk.len());
        let mut read = ReadBuf::new(&mut chunk[..length]);
        match Pin::new(&mut self.inner).poll_read(cx, &mut read) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
            Poll::Ready(Ok(())) => {}
        }
        for byte in read.filled() {
            // Charge the first byte, not just the delimiter: an unterminated
            // final line at EOF must not bypass the frame count. Empty lines
            // count too, since they still consume decoder work. Keep this state
            // in the reader so cancellation cannot reset a partial frame.
            if self.line_bytes == 0 {
                self.frames = self.frames.saturating_add(1);
                if self.frames > self.frame_limit {
                    self.failed = true;
                    return Poll::Ready(Err(io::Error::other("MCP wire limit")));
                }
            }
            self.total_bytes = self.total_bytes.saturating_add(1);
            self.line_bytes = self.line_bytes.saturating_add(1);
            if self.total_bytes > self.total_limit || self.line_bytes > self.line_limit {
                self.failed = true;
                return Poll::Ready(Err(io::Error::other("MCP wire limit")));
            }
            if *byte == b'\n' {
                self.line_bytes = 0;
            }
        }
        dest.put_slice(read.filled());
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn mcp_wire_limits_apply_to_unterminated_lines_and_total_stream() {
        for (bytes, line, total, passes) in [
            (&b"abc\nabc\n"[..], 4, 8, true),
            (&b"abcde"[..], 4, 8, false),
            (&b"abc\nabc\n"[..], 4, 7, false),
        ] {
            let mut reader = BoundedReader::new(bytes, line, total, 8);
            let mut output = Vec::new();
            assert_eq!(reader.read_to_end(&mut output).await.is_ok(), passes);
            assert!(output.len() <= total);
        }
    }

    #[tokio::test]
    async fn mcp_wire_limit_survives_fragmented_and_cancelled_reads() {
        let (mut writer, reader) = tokio::io::duplex(16);
        let mut reader = BoundedReader::new(reader, 4, 16, 8);
        writer.write_all(b"ab").await.unwrap();
        let mut first = [0; 2];
        reader.read_exact(&mut first).await.unwrap();
        let mut last = [0; 8];
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(1), reader.read(&mut last))
                .await
                .is_err()
        );
        writer.write_all(b"cde").await.unwrap();
        assert!(reader.read(&mut last).await.is_err());
        assert!(reader.read(&mut last).await.is_err());
    }

    #[tokio::test]
    async fn mcp_frame_count_bounds_small_frame_floods_and_unterminated_tail() {
        let limit = crate::mcp::MAX_MCP_SESSION_FRAMES;
        for frame in [&b"{}\n"[..], &b"{}\r\n"[..], &b"\n"[..]] {
            let bytes = frame.repeat(limit);
            let mut reader = BoundedReader::new(bytes.as_slice(), 32, 1_048_576, limit);
            let mut accepted = Vec::new();
            reader.read_to_end(&mut accepted).await.unwrap();
            assert_eq!(accepted, bytes);
            for tail in [frame, &b"{}"[..]] {
                let mut flood = bytes.clone();
                flood.extend_from_slice(tail);
                let mut reader = BoundedReader::new(flood.as_slice(), 32, 1_048_576, limit);
                let mut accepted = Vec::new();
                assert!(reader.read_to_end(&mut accepted).await.is_err());
                assert!(accepted.len() <= bytes.len());
            }
        }
    }

    #[tokio::test]
    async fn mcp_frame_count_survives_fragmented_and_cancelled_reads() {
        let (mut writer, reader) = tokio::io::duplex(64);
        let mut reader = BoundedReader::new(reader, 32, 1024, 2);
        writer.write_all(b"{}\n{").await.unwrap();
        let mut first = [0; 4];
        reader.read_exact(&mut first).await.unwrap();
        let mut next = [0; 16];
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(1), reader.read(&mut next))
                .await
                .is_err()
        );
        writer.write_all(b"}\n{}\n").await.unwrap();
        assert!(reader.read(&mut next).await.is_err());
        assert!(reader.read(&mut next).await.is_err());
    }
}
