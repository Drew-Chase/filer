use crate::auth::auth_middleware::Authentication;
use crate::filesystem::filesystem_data::{FilesystemData, FilesystemEntry};
use crate::helpers::http_error::{Error, Result};
use actix_web::web::{Bytes, Query};
use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, post, web};
use actix_web_lab::__reexports::futures_util::StreamExt;
use actix_web_lab::sse::{Data, Event, Sse};
use async_stream::stream;
use futures::Stream;
use serde_json::json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use sysinfo::Disks;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use uuid::uuid;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

// At module level
static UPLOAD_TRACKERS: OnceLock<UploadTracker> = OnceLock::new();

type UploadTracker = Arc<Mutex<HashMap<String, Sender<Event>>>>;

// Helper function to get or initialize the tracker
fn get_upload_trackers() -> &'static UploadTracker {
    UPLOAD_TRACKERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

#[get("/")]
async fn get_filesystem_entries(request: HttpRequest) -> Result<impl Responder> {
    let path = match request.headers().get("X-Filesystem-Path") {
        Some(header) => match header.to_str() {
            Ok(path_str) => PathBuf::from(path_str),
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "X-Filesystem-Path header is not a valid string"
                })));
            }
        },
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "X-Filesystem-Path header is missing"
            })));
        }
    };

    if cfg!(target_os = "windows") && (path.to_str() == Some("/") || path.to_str() == Some("")) {
        let disks = Disks::new_with_refreshed_list();

        let drives: Vec<FilesystemEntry> = disks
            .iter()
            .map(|disk| {
                let mount_point = disk.mount_point().to_path_buf();
                FilesystemEntry {
                    filename: mount_point.to_string_lossy().into_owned(),
                    path: mount_point.to_string_lossy().into_owned(),
                    size: disk.total_space(),
                    is_dir: true,
                    created: None,
                    last_modified: None,
                }
            })
            .collect();

        return Ok(HttpResponse::Ok().json(json!({
            "parent": None::<String>,
            "entries": drives
        })));
    }

    let entries: FilesystemData = path.try_into()?;
    Ok(HttpResponse::Ok().json(json!(entries)))
}

#[get("/download")]
async fn download(request: HttpRequest) -> Result<impl Responder> {
    // Check if X-Multiple-Paths header exists
    if let Some(multiple_paths_header) = request.headers().get("X-Multiple-Paths") {
        // Parse the JSON array of paths
        let paths_str = match multiple_paths_header.to_str() {
            Ok(paths_str) => paths_str,
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "X-Multiple-Paths header is not a valid string"
                })));
            }
        };

        let paths: Vec<String> = match serde_json::from_str(paths_str) {
            Ok(paths) => paths,
            Err(e) => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": format!("Failed to parse X-Multiple-Paths header as JSON array: {}", e)
                })));
            }
        };

        if paths.is_empty() {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "X-Multiple-Paths header must contain at least one path"
            })));
        }

        // Convert strings to PathBufs
        let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

        // Find the common base path for all paths
        let common_base = {
            // First, convert all paths to PathBufs for easier manipulation
            let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

            if path_bufs.is_empty() {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "No valid paths provided"
                })));
            }

            // Get the parent directory of the first path
            let first_path = &path_bufs[0];
            let mut common_parent = if first_path.is_file() {
                first_path.parent().unwrap_or(Path::new("")).to_path_buf()
            } else {
                first_path.clone()
            };

            // Find the common parent directory
            for path in &path_bufs[1..] {
                let path_parent = if path.is_file() {
                    path.parent().unwrap_or(Path::new("")).to_path_buf()
                } else {
                    path.clone()
                };

                // Find the common parent by comparing components
                let common_components: Vec<_> = common_parent
                    .components()
                    .zip(path_parent.components())
                    .take_while(|(a, b)| a == b)
                    .map(|(a, _)| a)
                    .collect();

                if common_components.is_empty() {
                    // No common parent, use the drive root
                    if let Some(drive) = common_parent.components().next() {
                        common_parent = PathBuf::from(drive.as_os_str());
                    } else {
                        common_parent = PathBuf::new();
                    }
                    break;
                } else {
                    // Rebuild the common parent from components
                    common_parent =
                        common_components
                            .iter()
                            .fold(PathBuf::new(), |mut path, component| {
                                path.push(component.as_os_str());
                                path
                            });
                }
            }

            // Convert to string and ensure it ends with a separator
            let mut common_prefix = common_parent.to_string_lossy().to_string();
            if !common_prefix.is_empty() && !common_prefix.ends_with('\\') {
                common_prefix.push('\\');
            }

            common_prefix
        };

        // Create a 100% in-memory streaming zip solution for multiple paths
        let data_stream: Pin<Box<dyn Stream<Item = Result<Bytes>>>> = Box::pin(stream! {
            // Create an in-memory buffer for the zip
            let buffer_size = 64 * 1024; // 64KB buffer
            let in_memory_buffer = std::io::Cursor::new(Vec::new());

            // Create a zip writer that writes to the in-memory buffer
            let mut zip = zip::ZipWriter::new(in_memory_buffer);
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated);

            // Use the common base path we found earlier
            let base_path = PathBuf::from(&common_base);
            let mut read_buffer = vec![0; buffer_size]; // 64KB read buffer

            // Keep track of directories we've already added
            let mut added_directories = std::collections::HashSet::<String>::new();

            // Process each path
            log::debug!("Processing {} paths", path_bufs.len());
            for (i, path) in path_bufs.iter().enumerate() {
                log::debug!("Processing path {}/{}: {}", i + 1, path_bufs.len(), path.display());
                if path.is_dir() {
                    log::debug!("Path is a directory: {}", path.display());
                    // Process directory
                    let dir = walkdir::WalkDir::new(path);
                    let mut file_count = 0;
                    for entry_result in dir {
                        let dir_entry = entry_result.map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to read directory: {}", e))))?;
                        let entry_path = dir_entry.path();
                        file_count += 1;
                        log::debug!("Found entry {}: {}", file_count, entry_path.display());

                        // Create a relative path
                        let entry_path_str = entry_path.to_string_lossy().replace('\\', "/");
                        let base_path_str = base_path.to_string_lossy().replace('\\', "/");
                        log::debug!("Entry path: {}", entry_path_str);
                        log::debug!("Base path: {}", base_path_str);

                        if entry_path_str.starts_with(&*base_path_str) {
                            let relative_path = &entry_path_str[base_path_str.len()..];
                            log::debug!("Relative path before processing: {}", relative_path);
                            // Replace backslashes with forward slashes and ensure no leading slash
                            let relative_path = relative_path.replace('\\', "/");
                            let relative_path = relative_path.trim_start_matches('/');
                            log::debug!("Relative path after processing: {}", relative_path);

                            if entry_path.is_dir() {
                                // Add directory entry to the zip
                                let dir_path = if relative_path.is_empty() {
                                    String::from("")
                                } else {
                                    format!("{}/", relative_path)
                                };

                                // Check if this directory has already been added
                                if !added_directories.contains(&dir_path) {
                                    log::debug!("Adding directory to zip: {}", dir_path);
                                    zip.add_directory(&dir_path, options)
                                        .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to create directory entry: {}", e))))?;
                                    added_directories.insert(dir_path);
                                } else {
                                    log::debug!("Directory already exists in zip: {}", dir_path);
                                }
                            } else if entry_path.is_file() {
                                // Create parent directory entries if needed
                                if let Some(parent) = Path::new(&relative_path).parent() {
                                    if !parent.as_os_str().is_empty() {
                                        let parent_path = parent.to_string_lossy().replace('\\', "/");
                                        if !parent_path.is_empty() {
                                            let dir_path = format!("{}/", parent_path);
                                            // Check if this directory has already been added
                                            if !added_directories.contains(&dir_path) {
                                                log::debug!("Adding parent directory to zip: {}", dir_path);
                                                zip.add_directory(&dir_path, options)
                                                    .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to create directory entry: {}", e))))?;
                                                added_directories.insert(dir_path);
                                            } else {
                                                log::debug!("Directory already exists in zip: {}", dir_path);
                                            }
                                        }
                                    }
                                }

                                // Start a new file in the zip
                                log::debug!("Adding file to zip: {}", relative_path);
                                zip.start_file(relative_path, options)
                                    .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to create zip entry: {}", e))))?;

                                // Open the file
                                let mut file = std::fs::File::open(entry_path)
                                    .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to open file {}: {}", entry_path.display(), e))))?;

                                // Read and write the file in chunks
                                let mut total_bytes_written = 0;
                                loop {
                                    let bytes_read = file.read(&mut read_buffer)
                                        .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to read file {}: {}", entry_path.display(), e))))?;

                                    if bytes_read == 0 { break; }

                                    zip.write_all(&read_buffer[..bytes_read])
                                        .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to write to zip: {}", e))))?;

                                    total_bytes_written += bytes_read;
                                }
                                log::debug!("Wrote {} bytes to zip for file: {}", total_bytes_written, relative_path);
                            }
                        }
                    }
                } else if path.is_file() {
                    log::debug!("Path is a file: {}", path.display());
                    // Process single file
                    // Create a relative path
                    let path_str = path.to_string_lossy();
                    let path_str = path_str.replace('\\', "/");

                    let base_path_str = base_path.to_string_lossy();
                    let base_path_str = base_path_str.replace('\\', "/");
                    let base_path_str = base_path_str.trim_end_matches('/');
                    log::debug!("Base path: {}", base_path_str);

                    if path_str.starts_with(&*base_path_str) {
                        let relative_path = &path_str[base_path_str.len()..];
                        log::debug!("Relative path before processing: {}", relative_path);
                        // Replace backslashes with forward slashes and ensure no leading slash
                        let relative_path = relative_path.replace('\\', "/");
                        let relative_path = relative_path.trim_start_matches('/');
                        log::debug!("Relative path after processing: {}", relative_path);

                        // Create parent directory entries if needed
                        if let Some(parent) = Path::new(&relative_path).parent() {
                            if !parent.as_os_str().is_empty() {
                                let parent_path = parent.to_string_lossy().replace('\\', "/");
                                if !parent_path.is_empty() {
                                    let dir_path = format!("{}/", parent_path);
                                    // Check if this directory has already been added
                                    if !added_directories.contains(&dir_path) {
                                        log::debug!("Adding parent directory to zip: {}", dir_path);
                                        zip.add_directory(&dir_path, options)
                                            .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to create directory entry: {}", e))))?;
                                        added_directories.insert(dir_path);
                                    } else {
                                        log::debug!("Directory already exists in zip: {}", dir_path);
                                    }
                                }
                            }
                        }

                        // Start a new file in the zip
                        log::debug!("Adding file to zip: {}", relative_path);
                        zip.start_file(relative_path, options)
                            .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to create zip entry: {}", e))))?;

                        // Open the file
                        let mut file = std::fs::File::open(path)
                            .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to open file {}: {}", path.display(), e))))?;

                        // Read and write the file in chunks
                        let mut total_bytes_written = 0;
                        loop {
                            let bytes_read = file.read(&mut read_buffer)
                                .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to read file {}: {}", path.display(), e))))?;

                            if bytes_read == 0 { break; }

                            zip.write_all(&read_buffer[..bytes_read])
                                .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to write to zip: {}", e))))?;

                            total_bytes_written += bytes_read;
                        }
                        log::debug!("Wrote {} bytes to zip for file: {}", total_bytes_written, relative_path);
                    }
                }
            }

            // Finish the zip file (writes the central directory)
            let cursor = zip.finish()
                .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to finish zip file: {}", e))))?;

            // Get the final zip data
            let zip_data = cursor.into_inner();

            // Log the size of the zip data
            log::debug!("Multi-path zip data size: {} bytes", zip_data.len());

            if zip_data.is_empty() {
                log::error!("Multi-path zip data is empty! No files were added to the archive.");
                // Add a dummy file to the zip so it's not empty
                let dummy_content = b"This is a dummy file to prevent the zip from being empty.";
                yield Ok(Bytes::from(dummy_content.to_vec()));
            } else {
                // Stream the zip data in chunks
                for (i, chunk) in zip_data.chunks(buffer_size).enumerate() {
                    log::debug!("Streaming chunk {}, size: {} bytes", i + 1, chunk.len());
                    yield Ok(Bytes::from(chunk.to_vec()));
                }
            }
        });

        return Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .insert_header((
                "Content-Disposition",
                format!(r#"attachment; filename="{}.zip""#, uuid::Uuid::new_v4()),
            ))
            .streaming(data_stream));
    }

    // If X-Multiple-Paths header doesn't exist, fall back to the original behavior
    let path = match request.headers().get("X-Filesystem-Path") {
        Some(header) => match header.to_str() {
            Ok(path_str) => PathBuf::from(path_str),
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "X-Filesystem-Path header is not a valid string"
                })));
            }
        },
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "X-Filesystem-Path header is missing"
            })));
        }
    };

    let entry: FilesystemEntry = path.clone().try_into()?;
    if entry.is_dir {
        // Create a 100% in-memory streaming zip solution
        let data_stream: Pin<Box<dyn Stream<Item = Result<Bytes>>>> = Box::pin(stream! {
            // Create an in-memory buffer for the zip
            let buffer_size = 64 * 1024; // 64KB buffer
            let in_memory_buffer = std::io::Cursor::new(Vec::new());

            // Create a zip writer that writes to the in-memory buffer
            let mut zip = zip::ZipWriter::new(in_memory_buffer);
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated);

            // Process files as we discover them
            let base_path = PathBuf::from(&entry.path);
            log::debug!("Single directory case - Base path: {}", base_path.display());
            let mut read_buffer = vec![0; buffer_size]; // 64KB read buffer

            // Keep track of directories we've already added
            let mut added_directories = std::collections::HashSet::<String>::new();

            // Process files directly from the directory walker
            log::debug!("Walking directory: {}", entry.path);
            let dir = walkdir::WalkDir::new(&entry.path);
            let mut file_count = 0;
            for entry_result in dir {
                let dir_entry = entry_result.map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to read directory: {}", e))))?;
                let path = dir_entry.path();
                file_count += 1;
                log::debug!("Found entry {}: {}", file_count, path.display());

                // Process both files and directories
                log::debug!("Checking if path can be stripped of base path: {} vs {}", path.display(), base_path.display());
                if let Ok(relative_path) = path.strip_prefix(&base_path) {
                    log::debug!("Relative path after strip_prefix: {}", relative_path.display());
                    // Convert to string and ensure forward slashes
                    let rel_path_str = relative_path.to_string_lossy().replace('\\', "/");
                    log::debug!("Relative path after processing: {}", rel_path_str);

                    if path.is_dir() {
                        // Add directory entry to the zip
                        let dir_path = if rel_path_str.is_empty() {
                            String::from("")
                        } else {
                            format!("{}/", rel_path_str)
                        };

                        // Check if this directory has already been added
                        if !added_directories.contains(&dir_path) {
                            log::debug!("Adding directory to zip: {}", dir_path);
                            zip.add_directory(&dir_path, options)
                                .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to create directory entry: {}", e))))?;
                            added_directories.insert(dir_path);
                        } else {
                            log::debug!("Directory already exists in zip: {}", dir_path);
                        }
                    } else if path.is_file() {
                        // Create parent directory entries if needed
                        if let Some(parent) = Path::new(&rel_path_str).parent() {
                            if !parent.as_os_str().is_empty() {
                                let parent_path = parent.to_string_lossy().replace('\\', "/");
                                if !parent_path.is_empty() {
                                    let dir_path = format!("{}/", parent_path);
                                    // Check if this directory has already been added
                                    if !added_directories.contains(&dir_path) {
                                        log::debug!("Adding parent directory to zip: {}", dir_path);
                                        zip.add_directory(&dir_path, options)
                                            .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to create directory entry: {}", e))))?;
                                        added_directories.insert(dir_path);
                                    } else {
                                        log::debug!("Directory already exists in zip: {}", dir_path);
                                    }
                                }
                            }
                        }

                        // Start a new file in the zip
                        log::debug!("Adding file to zip: {}", rel_path_str);
                        let rel_path_for_zip = rel_path_str.clone();
                        zip.start_file(rel_path_for_zip, options)
                            .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to create zip entry: {}", e))))?;

                        // Open the file
                        let mut file = std::fs::File::open(path)
                            .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to open file {}: {}", path.display(), e))))?;

                        // Read and write the file in chunks
                        let mut total_bytes_written = 0;
                        loop {
                            let bytes_read = file.read(&mut read_buffer)
                                .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to read file {}: {}", path.display(), e))))?;

                            if bytes_read == 0 { break; }

                            zip.write_all(&read_buffer[..bytes_read])
                                .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to write to zip: {}", e))))?;

                            total_bytes_written += bytes_read;
                        }
                        log::debug!("Wrote {} bytes to zip for file: {}", total_bytes_written, rel_path_str);
                    }
                }
            }

            // Finish the zip file (writes the central directory)
            let cursor = zip.finish()
                .map_err(|e| Error::InternalError(anyhow::Error::msg(format!("Failed to finish zip file: {}", e))))?;

            // Get the final zip data
            let zip_data = cursor.into_inner();

            // Log the size of the zip data
            log::debug!("Single directory zip data size: {} bytes", zip_data.len());

            if zip_data.is_empty() {
                log::error!("Single directory zip data is empty! No files were added to the archive.");
                // Add a dummy file to the zip so it's not empty
                let dummy_content = b"This is a dummy file to prevent the zip from being empty.";
                yield Ok(Bytes::from(dummy_content.to_vec()));
            } else {
                // Stream the zip data in chunks
                for (i, chunk) in zip_data.chunks(buffer_size).enumerate() {
                    log::debug!("Streaming chunk {}, size: {} bytes", i + 1, chunk.len());
                    yield Ok(Bytes::from(chunk.to_vec()));
                }
            }
        });

        Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .insert_header((
                "Content-Disposition",
                format!(r#"attachment; filename="{}.zip""#, entry.filename),
            ))
            .streaming(data_stream))
    } else {
        // stream file.
        let file = std::fs::File::open(&entry.path).map_err(|_| {
            Error::InternalError(anyhow::Error::msg(format!(
                "Failed to open file: {}",
                entry.path
            )))
        })?;

        let data_stream = stream! {
            let mut chunk = vec![0u8;10*1024*1024];
            let mut file = file;
            loop {
                match file.read(&mut chunk) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 { break; }
                        yield Result::<Bytes>::Ok(Bytes::from(chunk[..bytes_read].to_vec()));
                    },
                    Err(e) => {
                        yield Result::<Bytes>::Err(Error::InternalError(anyhow::Error::msg(format!(
                            "Failed to read file: {}", e
                        ))));
                        break;
                    }
                }
            }
        };
        Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .insert_header((
                "Content-Disposition",
                format!(r#"attachment; filename="{}""#, entry.filename),
            ))
            .streaming(data_stream))
    }
}

#[get("search")]
async fn search(query_map: Query<HashMap<String, String>>) -> Result<impl Responder> {
    if let Some(_query) = query_map.get("q") {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().json(json!({
            "error": "Search query is required"
        })))
    }
}
// Add a new endpoint for progress tracking
#[get("/upload/progress/{upload_id}")]
async fn upload_progress(upload_id: web::Path<String>) -> impl Responder {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Store the sender in our tracker
    {
        let mut trackers = get_upload_trackers().lock().await;
        trackers.insert(upload_id.to_string(), tx);
    }

    Sse::from_infallible_receiver(rx).with_keep_alive(Duration::from_secs(3))
}

#[post("/upload")]
async fn upload(mut payload: web::Payload, request: HttpRequest) -> impl Responder {
    // Extract upload ID
    let upload_id = match request.headers().get("X-Upload-ID") {
        Some(header) => match header.to_str() {
            Ok(id) => id.to_string(),
            Err(_) => {
                return HttpResponse::BadRequest().json(json!({
                    "error": "Invalid X-Upload-ID header"
                }));
            }
        },
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "X-Upload-ID header is required"
            }));
        }
    };

    let path = match request.headers().get("X-Filesystem-Path") {
        Some(header) => match header.to_str() {
            Ok(path_str) => PathBuf::from(path_str),
            Err(_) => {
                return HttpResponse::BadRequest().json(json!({
                    "error": "Invalid X-Filesystem-Path header"
                }));
            }
        },
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "X-Filesystem-Path header is required"
            }));
        }
    };

    // Get the progress sender for this upload
    let progress_sender = {
        let trackers = get_upload_trackers().lock().await;
        trackers.get(&upload_id).cloned()
    };

    let mut file = match tokio::fs::File::create(&path).await {
        Ok(file) => file,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create file"
            }));
        }
    };

    let mut total_bytes = 0u64;

    // Process the upload
    while let Some(chunk) = payload.next().await {
        match chunk {
            Ok(bytes) => {
                if file.write_all(&bytes).await.is_err() {
                    return HttpResponse::InternalServerError().json(json!({
                        "error": "Failed to write file"
                    }));
                }

                total_bytes += bytes.len() as u64;

                // Send progress update if we have a sender
                if let Some(sender) = &progress_sender {
                    let _ = sender
                        .send(Event::from(Data::new(
                            json!({
                                "status": "progress",
                                "bytesUploaded": total_bytes
                            })
                            .to_string(),
                        )))
                        .await;
                }
            }
            Err(_) => {
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to read upload data"
                }));
            }
        }
    }

    // Send completion event
    if let Some(sender) = progress_sender {
        let _ = sender
            .send(Event::from(Data::new(
                json!({
                    "status": "complete",
                    "bytesUploaded": total_bytes
                })
                .to_string(),
            )))
            .await;

        // Clean up the tracker
        let mut trackers = get_upload_trackers().lock().await;
        trackers.remove(&upload_id);
    }

    HttpResponse::Ok().json(json!({
        "status": "success",
        "bytesUploaded": total_bytes
    }))
}

#[post("/copy")]
async fn copy_filesystem_entry(request: HttpRequest) -> Result<impl Responder> {
    // Extract a source path
    let source_path = match request.headers().get("X-Filesystem-Path") {
        Some(header) => match header.to_str() {
            Ok(path_str) => PathBuf::from(path_str),
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "X-Filesystem-Path header is not a valid string"
                })));
            }
        },
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "X-Filesystem-Path header is missing"
            })));
        }
    };

    // Extract destination path
    let dest_path = match request.headers().get("X-NewFilesystem-Path") {
        Some(header) => match header.to_str() {
            Ok(path_str) => PathBuf::from(path_str),
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "X-NewFilesystem-Path header is not a valid string"
                })));
            }
        },
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "X-NewFilesystem-Path header is missing"
            })));
        }
    };

    // Verify source exists
    if !source_path.exists() {
        return Ok(HttpResponse::NotFound().json(json!({
            "error": "Source path does not exist"
        })));
    }

    // Copy the filesystem entry
    if source_path.is_dir() {
        // Create a copy function for recursive directory copy
        fn copy_dir_all(
            src: impl AsRef<std::path::Path>,
            dst: impl AsRef<std::path::Path>,
        ) -> std::io::Result<()> {
            std::fs::create_dir_all(&dst)?;
            for entry in std::fs::read_dir(src)? {
                let entry = entry?;
                let ty = entry.file_type()?;
                let src_path = entry.path();
                let dst_path = dst.as_ref().join(entry.file_name());

                if ty.is_dir() {
                    copy_dir_all(&src_path, &dst_path)?;
                } else {
                    std::fs::copy(&src_path, &dst_path)?;
                }
            }
            Ok(())
        }

        if let Err(e) = copy_dir_all(&source_path, &dest_path) {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to copy directory: {}", e)
            })));
        }
    } else {
        // Copy file
        if let Err(e) = std::fs::copy(&source_path, &dest_path) {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to copy file: {}", e)
            })));
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Entry copied successfully"
    })))
}

#[post("/move")]
async fn move_filesystem_entry(request: HttpRequest) -> Result<impl Responder> {
    // Extract a source path
    let source_path = match request.headers().get("X-Filesystem-Path") {
        Some(header) => match header.to_str() {
            Ok(path_str) => PathBuf::from(path_str),
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "X-Filesystem-Path header is not a valid string"
                })));
            }
        },
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "X-Filesystem-Path header is missing"
            })));
        }
    };

    // Extract destination path
    let dest_path = match request.headers().get("X-NewFilesystem-Path") {
        Some(header) => match header.to_str() {
            Ok(path_str) => PathBuf::from(path_str),
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "X-NewFilesystem-Path header is not a valid string"
                })));
            }
        },
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "X-NewFilesystem-Path header is missing"
            })));
        }
    };

    // Verify source exists
    if !source_path.exists() {
        return Ok(HttpResponse::NotFound().json(json!({
            "error": "Source path does not exist"
        })));
    }

    // Move/rename is the same operation in filesystem terms
    if let Err(e) = std::fs::rename(&source_path, &dest_path) {
        return Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to move entry: {}", e)
        })));
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Entry moved successfully"
    })))
}

#[delete("/")]
async fn delete_filesystem_entry(request: HttpRequest) -> Result<impl Responder> {
    let path = match request.headers().get("X-Filesystem-Path") {
        Some(header) => match header.to_str() {
            Ok(path_str) => PathBuf::from(path_str),
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "X-Filesystem-Path header is not a valid string"
                })));
            }
        },
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "X-Filesystem-Path header is missing"
            })));
        }
    };

    // Verify a path exists
    if !path.exists() {
        return Ok(HttpResponse::NotFound().json(json!({
            "error": "Path does not exist"
        })));
    }

    // Delete a file or directory
    if path.is_dir() {
        if let Err(e) = std::fs::remove_dir_all(&path) {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to delete directory: {}", e)
            })));
        }
    } else if let Err(e) = std::fs::remove_file(&path) {
        return Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to delete file: {}", e)
        })));
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Entry deleted successfully"
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/filesystem")
            .wrap(Authentication::new())
            .service(get_filesystem_entries)
            .service(download)
            .service(search)
            .service(upload)
            .service(upload_progress)
            .service(copy_filesystem_entry)
            .service(move_filesystem_entry)
            .service(delete_filesystem_entry),
    );
}
