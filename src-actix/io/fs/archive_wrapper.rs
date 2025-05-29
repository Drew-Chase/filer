use actix_web_lab::sse;
use actix_web_lab::sse::Event;
use anyhow::Result;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use tokio::fs;

pub async fn archive(archive_path: impl AsRef<Path>, entries: Vec<PathBuf>, sender: tokio::sync::mpsc::Sender<Event>) -> Result<()> {
    let file = fs::File::create(archive_path.as_ref()).await?;
    let file = file.into_std().await;
    let mut archive = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    // Calculate total bytes to process
    let mut total_bytes: u64 = 0;
    let mut processed_bytes: u64 = 0;
    
    // First, calculate total bytes
    for entry in &entries {
        if entry.is_dir() {
            let dirs = walkdir::WalkDir::new(entry);
            for dir_entry in dirs.into_iter() {
                if let Ok(dir_entry) = dir_entry {
                    if dir_entry.path().is_file() {
                        if let Ok(metadata) = std::fs::metadata(dir_entry.path()) {
                            total_bytes += metadata.len();
                        }
                    }
                }
            }
        } else if let Ok(metadata) = std::fs::metadata(entry) {
            total_bytes += metadata.len();
        }
    }
    
    // Send initial progress
    let _ = sender.send(Event::from(sse::Data::new(format!("{{ \"progress\": {:.1} }}", 0.0)))).await;
    
    // Process the files and update progress
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
                    
                    // Use a custom buffer to track progress
                    let mut buffer = [0; 4096]; // 4KB buffer
                    let mut last_progress_update = std::time::Instant::now();
                    
                    loop {
                        let bytes_read = match reader.read(&mut buffer) {
                            Ok(0) => break, // EOF
                            Ok(n) => n,
                            Err(e) => return Err(anyhow::anyhow!("Error reading file: {}", e)),
                        };
                        
                        archive.write_all(&buffer[..bytes_read])?;
                        
                        processed_bytes += bytes_read as u64;
                        
                        // Send progress update with rate limiting (max once per 100ms)
                        let now = std::time::Instant::now();
                        if now.duration_since(last_progress_update).as_millis() > 100 {
                            let progress = if total_bytes > 0 {
                                (processed_bytes as f32 / total_bytes as f32) * 100.0
                            } else {
                                0.0
                            };
                            
                            let _ = sender.send(Event::from(sse::Data::new(
                                format!("{{ \"progress\": {:.1} }}", progress)
                            ))).await;
                            
                            last_progress_update = now;
                        }
                    }
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
            
            // Use a custom buffer to track progress
            let mut buffer = [0; 4096]; // 4KB buffer
            let mut last_progress_update = std::time::Instant::now();
            
            loop {
                let bytes_read = match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => n,
                    Err(e) => return Err(anyhow::anyhow!("Error reading file: {}", e)),
                };
                
                archive.write_all(&buffer[..bytes_read])?;
                
                processed_bytes += bytes_read as u64;
                
                // Send progress update with rate limiting (max once per 100ms)
                let now = std::time::Instant::now();
                if now.duration_since(last_progress_update).as_millis() > 100 {
                    let progress = if total_bytes > 0 {
                        (processed_bytes as f32 / total_bytes as f32) * 100.0
                    } else {
                        0.0
                    };
                    
                    let _ = sender.send(Event::from(sse::Data::new(
                        format!("{{ \"progress\": {:.1} }}", progress)
                    ))).await;
                    
                    last_progress_update = now;
                }
            }
        }
    }

    // Send the completion message
    let _ = sender.send(Event::from(sse::Data::new("{ \"progress\": 100.0, \"status\": \"complete\" }"))).await;
    
    archive.finish()?;
    Ok(())
}