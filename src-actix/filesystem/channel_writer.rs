use bytes::Bytes;
use std::io::{self, Seek, Write};
use tokio::sync::mpsc;
pub struct ChannelWriter {
    tx: mpsc::Sender<Result<Bytes, io::Error>>,
    buffer: Vec<u8>,
}
impl ChannelWriter {
    pub(crate) fn new(tx: mpsc::Sender<Result<Bytes, io::Error>>) -> Self {
        Self {
            tx,
            buffer: Vec::with_capacity(u16::MAX as usize),
        }
    }
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        if self.buffer.len() >= u16::MAX as usize {
            let to_send = std::mem::take(&mut self.buffer);
            self.tx
                .blocking_send(Ok(Bytes::from(to_send)))
                .map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, format!("pipe send error: {e}"))
                })?;
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            let to_send = std::mem::take(&mut self.buffer);
            self.tx
                .blocking_send(Ok(Bytes::from(to_send)))
                .map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, format!("pipe flush error: {e}"))
                })?;
        }
        Ok(())
    }
}

impl Seek for ChannelWriter {
    fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
        Ok(0)
    }
}
