use crate::io::streams::channel_writer::ChannelWriter;
use anyhow::Result;
use bytes::Bytes;
use log::debug;
use std::io;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{CompressionMethod, ZipWriter};

pub struct ZipChannelWriter {
    zip_writer: ZipWriter<ChannelWriter>,
    options: SimpleFileOptions,
}

impl ZipChannelWriter {
    pub fn new(sender: tokio::sync::mpsc::Sender<std::result::Result<Bytes, io::Error>>) -> Self {
        let channel_writer = ChannelWriter::new(sender);
        let mut zip_writer = ZipWriter::new(channel_writer);

        // Set the compression method to the simplest,
        // This should improve speeds at the cost of file size.
        let options: SimpleFileOptions =
            FileOptions::default().compression_method(CompressionMethod::Stored);

        // This will prevent the stream from attempting to seek backwards
        zip_writer.set_flush_on_finish_file(true);

        ZipChannelWriter {
            zip_writer,
            options,
        }
    }

    pub fn add_directory_entry(
        &mut self,
        path: impl AsRef<Path>,
        entry: impl AsRef<str>,
    ) -> Result<()> {
        let entry = entry.as_ref();
        let path = path.as_ref();
        debug!(
            "Adding directory to zip archive: {} -> {}",
            path.display(),
            entry
        );

        self.zip_writer.add_directory(entry, self.options)?;
        let sub_paths = walkdir::WalkDir::new(path);
        for sub_path in sub_paths.into_iter().flatten() {
            let path = sub_path.path();
            let entry = format!("{}/{}", entry, sub_path.file_name().to_str().unwrap());
            if path.is_dir() {
                self.add_directory_entry(path, entry)?;
            } else {
                self.add_file_entry(path, entry)?;
            }
        }

        Ok(())
    }

    pub fn add_file_entry(&mut self, path: impl AsRef<Path>, entry: impl AsRef<str>) -> Result<()> {
        let path = path.as_ref();
        let entry = entry.as_ref();

        debug!(
            "Adding file to zip archive: {} -> {}",
            path.display(),
            entry
        );
        self.zip_writer.start_file(entry, self.options)?;
        let mut read_buffer = [0; 64 * 1024]; // 64KB buffer size
        let mut file = std::fs::File::open(path)?;

        loop {
            let bytes_read = file.read(&mut read_buffer)?;
            if bytes_read == 0 {
                break;
            }
            self.zip_writer.write_all(&read_buffer[..bytes_read])?;
        }

        Ok(())
    }

    pub fn finish(self) -> Result<()> {
        let mut channel_writer: ChannelWriter = self.zip_writer.finish()?;
        channel_writer.flush()?;

        Ok(())
    }
}
