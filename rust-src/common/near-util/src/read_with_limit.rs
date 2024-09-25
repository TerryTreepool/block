
pub struct ReadWithLimit {
    limit_orig: u64,
    limit: u64,
    reader: Box<dyn async_std::io::Read + Send + Sync + Unpin>,
}

impl async_std::io::Read for ReadWithLimit {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        if self.limit == 0 {
            return std::task::Poll::Ready(Ok(0));
        }

        let min = std::cmp::min(buf.len() as u64, self.limit) as usize;
        let ret =
            std::pin::Pin::new(self.reader.as_mut()).poll_read(cx, &mut buf[..min]);
        match ret {
            std::task::Poll::Ready(Ok(n)) => {
                self.limit -= n as u64;
                std::task::Poll::Ready(Ok(n))
            }
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e)),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl ReadWithLimit {
    pub fn new(limit: u64, reader: Box<dyn async_std::io::Read + Send + Sync + Unpin>) -> Self {
        Self {
            limit,
            limit_orig: limit,
            reader,
        }
    }

    pub fn reset(&mut self) {
        self.limit = self.limit_orig;
    }
}


/// ReadBuf
pub struct BufReadWithLimit {
    limit_orig: u64,
    limit: u64,
    reader: Box<dyn async_std::io::BufRead + Send + Sync + Unpin>,
}

impl async_std::io::Read for BufReadWithLimit {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        if self.limit == 0 {
            return std::task::Poll::Ready(Ok(0));
        }

        let min = std::cmp::min(buf.len() as u64, self.limit) as usize;
        let ret =
            std::pin::Pin::new(self.reader.as_mut()).poll_read(cx, &mut buf[..min]);
        match ret {
            std::task::Poll::Ready(Ok(n)) => {
                self.limit -= n as u64;
                std::task::Poll::Ready(Ok(n))
            }
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e)),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl BufReadWithLimit {
    pub fn new(limit: u64, reader: Box<dyn async_std::io::BufRead + Send + Sync + Unpin>) -> Self {
        Self {
            limit,
            limit_orig: limit,
            reader,
        }
    }

    pub fn reset(&mut self) {
        self.limit = self.limit_orig;
    }
}
