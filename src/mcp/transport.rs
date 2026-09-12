//! Apply byte limits before the SDK buffers or decodes a JSON-RPC line.

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
    failed: bool,
}

impl<R> BoundedReader<R> {
    pub(super) fn new(inner: R, line_limit: usize, total_limit: usize) -> Self {
        Self {
            inner,
            line_bytes: 0,
            total_bytes: 0,
            line_limit,
            total_limit,
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
            return Poll::Ready(Err(io::Error::other("MCP wire byte limit")));
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
            self.total_bytes = self.total_bytes.saturating_add(1);
            self.line_bytes = self.line_bytes.saturating_add(1);
            if self.total_bytes > self.total_limit || self.line_bytes > self.line_limit {
                self.failed = true;
                return Poll::Ready(Err(io::Error::other("MCP wire byte limit")));
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
            let mut reader = BoundedReader::new(bytes, line, total);
            let mut output = Vec::new();
            assert_eq!(reader.read_to_end(&mut output).await.is_ok(), passes);
            assert!(output.len() <= total);
        }
    }

    #[tokio::test]
    async fn mcp_wire_limit_survives_fragmented_and_cancelled_reads() {
        let (mut writer, reader) = tokio::io::duplex(16);
        let mut reader = BoundedReader::new(reader, 4, 16);
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
}
