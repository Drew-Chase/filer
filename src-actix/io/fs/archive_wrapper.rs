use anyhow::Result;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn archive(archive_path: impl AsRef<Path>, entries: Vec<PathBuf>) -> Result<()> {
    // Use tokio's async file operations
    let file = fs::File::create(archive_path.as_ref()).await?;
    // Convert to sync for zip compatibility
    let file = file.into_std().await;
    let mut archive = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    for entry in entries {
        if entry.is_dir() {
            // Get directory name for the root path in the archive
            let dir_name = entry
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?;

            // Add the root directory with its name
            let dir_path = dir_name.to_string_lossy();
            archive.add_directory(&*dir_path, options)?;

            // Process files in the directory
            let dirs = walkdir::WalkDir::new(&entry);
            for dir_entry in dirs.into_iter() {
                let dir_entry = dir_entry?;
                let path = dir_entry.path();

                // Create proper paths in the archive that include the parent directory name
                if path == entry {
                    // Skip the root - we already added it
                    continue;
                }

                // Get the relative path from the source root
                let rel_path = path.strip_prefix(&entry)?;
                // Create the full path in the archive by combining dir_name with rel_path
                let archive_path = Path::new(&*dir_path).join(rel_path);
                let archive_path_str = archive_path.to_string_lossy();

                if path.is_file() {
                    archive.start_file(archive_path_str, options)?;

                    // Use buffered reading with a reasonable buffer size
                    let file = std::fs::File::open(path)?;
                    let mut reader = BufReader::with_capacity(8192, file);

                    // Stream the file in chunks
                    io::copy(&mut reader, &mut archive)?;
                } else if path.is_dir() {
                    archive.add_directory(archive_path_str, options)?;
                }
            }
        } else {
            // For individual files, preserve parent directories if they exist
            let rel_path = entry.file_name()
                                .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

            let archive_path = rel_path.to_string_lossy();
            archive.start_file(archive_path, options)?;

            // Use buffered reading with a reasonable buffer size
            let file = std::fs::File::open(&entry)?;
            let mut reader = BufReader::with_capacity(8192, file);

            // Stream the file in chunks instead of loading it all into memory
            io::copy(&mut reader, &mut archive)?;
        }
    }

    archive.finish()?;
    Ok(())
}