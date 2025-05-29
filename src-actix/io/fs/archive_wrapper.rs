use anyhow::Result;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use actix_web_lab::sse;
use actix_web_lab::sse::{Event};
use tokio::fs;

pub async fn archive(archive_path: impl AsRef<Path>, entries: Vec<PathBuf>, sender: tokio::sync::mpsc::Sender<Event>) -> Result<()> {
    let file = fs::File::create(archive_path.as_ref()).await?;
    let file = file.into_std().await;
    let mut archive = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    // Calculate total files to process
    let mut total_files = 0;
    let mut processed_files = 0;
    
    // First count all files
    for entry in &entries {
        if entry.is_dir() {
            let dirs = walkdir::WalkDir::new(entry);
            for dir_entry in dirs.into_iter() {
                if let Ok(dir_entry) = dir_entry {
                    if dir_entry.path().is_file() {
                        total_files += 1;
                    }
                }
            }
        } else {
            total_files += 1;
        }
    }
    
    // Now process the files and update progress
    for entry in entries {
        if entry.is_dir() {
            let dir_name = entry
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?;

            let dir_path = dir_name.to_string_lossy();
            archive.add_directory(&*dir_path, options)?;

            let dirs = walkdir::WalkDir::new(&entry);
            for dir_entry in dirs.into_iter() {
                let dir_entry = dir_entry?;
                let path = dir_entry.path();

                if path == entry {
                    continue;
                }

                let rel_path = path.strip_prefix(&entry)?;
                let archive_path = Path::new(&*dir_path).join(rel_path);
                let archive_path_str = archive_path.to_string_lossy();

                if path.is_file() {
                    archive.start_file(archive_path_str, options)?;

                    let file = std::fs::File::open(path)?;
                    let mut reader = BufReader::with_capacity(8192, file);

                    io::copy(&mut reader, &mut archive)?;
                    
                    // Update progress
                    processed_files += 1;
                    let progress = (processed_files as f32 / total_files as f32) * 100.0;
                    
                    // Send progress update via SSE
                    let _ = sender.send(Event::from(sse::Data::new(format!("{{ \"progress\": {:.1} }}", progress)))).await;
                } else if path.is_dir() {
                    archive.add_directory(archive_path_str, options)?;
                }
            }
        } else {
            let rel_path = entry.file_name()
                                .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

            let archive_path = rel_path.to_string_lossy();
            archive.start_file(archive_path, options)?;

            let file = std::fs::File::open(&entry)?;
            let mut reader = BufReader::with_capacity(8192, file);

            io::copy(&mut reader, &mut archive)?;
            
            // Update progress
            processed_files += 1;
            let progress = (processed_files as f32 / total_files as f32) * 100.0;
            
            // Send progress update via SSE
            let _ = sender.send(Event::from(sse::Data::new(format!("{{ \"progress\": {:.1} }}", progress)))).await;
        }
    }

    // Send the completion message
    let _ = sender.send(Event::from(sse::Data::new("{ \"progress\": 100.0, \"status\": \"complete\" }"))).await;
    
    archive.finish()?;
    Ok(())
}