use bytes::Bytes;
use std::io::{self, Read, Seek, SeekFrom, Write};
use tokio::sync::mpsc;
pub struct ChannelWriter {
    tx: mpsc::Sender<Result<Bytes, io::Error>>,
    buffer: Vec<u8>,
    pos: u64,
}
impl ChannelWriter {
    pub(crate) fn new(tx: mpsc::Sender<Result<Bytes, io::Error>>) -> Self {
        Self {
            tx,
            buffer: Vec::with_capacity(u16::MAX as usize),
            pos: 0,
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
impl Read for ChannelWriter {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "ChannelWriter is write-only"))
    }
}
impl Seek for ChannelWriter {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::Current(offset) => (self.pos as i64 + offset) as u64,
            SeekFrom::End(offset) => (self.pos as i64 + offset) as u64,
        };
        self.pos = new_pos;
        Ok(self.pos)
    }
}

