use crate::auth::auth_middleware::Authentication;
use crate::helpers::http_error::Result;
use crate::io::fs::archive_wrapper;
use crate::io::fs::download_parameters::DownloadParameters;
use crate::io::fs::filesystem_data::{FilesystemData, FilesystemEntry};
use crate::io::fs::indexer::indexer_data;
use crate::io::fs::indexer::indexer_data::IndexerData;
use actix_web::http::header::ContentDisposition;
use actix_web::web::Query;
use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, post, web};
use actix_web_lab::__reexports::futures_util::StreamExt;
use actix_web_lab::sse::{Data, Event, Sse};
use log::*;
use serde_json::json;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use sysinfo::Disks;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::duplex;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio_util::io::ReaderStream;

// At module level
static UPLOAD_TRACKERS: OnceLock<FileProcessTracker> = OnceLock::new();
static ARCHIVE_TRACKERS: OnceLock<FileProcessTracker> = OnceLock::new();

type FileProcessTracker = Arc<Mutex<HashMap<String, Sender<Event>>>>;

// Helper function to get or initialize the tracker
fn get_upload_trackers() -> &'static FileProcessTracker {
    UPLOAD_TRACKERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}
fn get_archive_trackers() -> &'static FileProcessTracker {
    ARCHIVE_TRACKERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

#[get("/")]
async fn get_filesystem_entries(request: HttpRequest) -> Result<impl Responder> {
    let mut path = match request.headers().get("X-Filesystem-Path") {
        Some(header) => match header.to_str() {
            Ok(path_str) => { 

                PathBuf::from(path_str) 
            },
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

    // Handle root or empty path
    if path.to_str() == Some("/") || path.to_str() == Some("") {
        #[cfg(target_os = "windows")]
        {
            // On Windows, show available drives
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

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix systems, use the root directory "/"
            path = PathBuf::from("/");
            // Continue to the normal path handling below
        }
    }

    let entries: FilesystemData = path.try_into()?;
    Ok(HttpResponse::Ok().json(json!(entries)))
}

#[get("/download")]
async fn download(query: Query<DownloadParameters>) -> Result<impl Responder> {
    use archflow::compress::FileOptions;
    use archflow::compress::tokio::archive::ZipArchive;
    use archflow::compression::CompressionMethod;
    use archflow::error::ArchiveError;
    use archflow::types::FileDateTime;
    let items: Vec<PathBuf> = query
        .items
        .iter()
        .map(|item| Path::new(format!("{}{}", query.cwd, item).as_str()).to_path_buf())
        .collect();
    let is_single_entry = items.len() == 1;
    let is_single_entry_directory = is_single_entry && items[0].is_dir();

    let filename: String = if is_single_entry {
        let guid = uuid::Uuid::new_v4().to_string();
        let name = items[0]
            .file_name()
            .unwrap_or(OsStr::new(&guid))
            .to_string_lossy()
            .into_owned();
        if is_single_entry_directory {
            format!("{}.zip", name)
        } else {
            name.to_string()
        }
    } else {
        format!("{}.zip", uuid::Uuid::new_v4())
    };

    // If there is only one entry, and it's a file,
    // then stream the individual file to the client.
    if is_single_entry && !is_single_entry_directory {
        let filepath = items[0].clone();
        debug!("Downloading single file: {}", filepath.display());

        let file = File::open(&filepath)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let stream = ReaderStream::new(file);

        return Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .insert_header(ContentDisposition::attachment(filename))
            .streaming(stream));
    }

    // For directories or multiple files, create a zip archive
    let (w, r) = duplex(4096);
    let cwd = query.cwd.clone();
    let items = items.clone();

    tokio::spawn(async move {
        let mut archive = ZipArchive::new_streamable(w);
        let options = FileOptions::default()
            .last_modified_time(FileDateTime::Now)
            .compression_method(CompressionMethod::Store());

        // Collect all files paths to put in the zip
        let items_to_write = if is_single_entry_directory {
            match tokio::fs::read_dir(items[0].clone()).await {
                Ok(mut dir) => {
                    let mut paths = Vec::new();
                    while let Ok(Some(entry)) = dir.next_entry().await {
                        paths.push(entry.path());
                    }
                    paths
                }
                Err(_) => items,
            }
        } else {
            items
        };

        for item in items_to_write {
            if let Some(filename) = item.file_name() {
                let filename = filename.to_string_lossy().into_owned();
                if item.is_dir() {
                    // Process directory
                    let walker = walkdir::WalkDir::new(&item);
                    if let Err(e) = archive.append_directory(filename.as_str(), &options).await {
                        error!("Failed to add directory to zip archive: {}", e);
                        continue;
                    }

                    for entry in walker.into_iter().flatten() {
                        let path = entry.path();
                        let relative_path = path.strip_prefix(&cwd).unwrap_or(path);

                        if path.is_dir() {
                            debug!(
                                "Adding directory to zip archive: {} -> {}",
                                path.display(),
                                relative_path.display()
                            );
                            if let Err(e) = archive
                                .append_directory(
                                    relative_path.to_string_lossy().replace('\\', "/").as_ref(),
                                    &options,
                                )
                                .await
                            {
                                error!("Failed to add directory to zip archive: {}", e);
                                continue;
                            }
                            continue; // Directories are automatically created when adding files
                        }

                        debug!(
                            "Adding file to zip archive: {} -> {}",
                            path.display(),
                            relative_path.display()
                        );
                        if let Ok(mut file) = File::open(path).await {
                            let _ = archive
                                .append(
                                    relative_path.to_string_lossy().replace('\\', "/").as_ref(),
                                    &options,
                                    &mut file,
                                )
                                .await;
                        }
                    }
                } else {
                    // Process a single file
                    debug!(
                        "Adding file to zip archive: {} -> {}",
                        item.display(),
                        filename
                    );
                    if let Ok(mut file) = File::open(&item).await {
                        if let Err(e) = archive.append(filename.as_str(), &options, &mut file).await
                        {
                            if matches!(&e, ArchiveError::IoError(err) if err.kind() == ErrorKind::BrokenPipe)
                            {
                                warn!(
                                    "Zip archive stream closed, this is most-likely due to the client closing the connection."
                                );
                                break;
                            }
                            error!("Failed to add file to zip archive: {}", e);
                            continue;
                        }
                    }
                }
            }
        }

        let _ = archive.finalize().await;
    });

    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .insert_header(ContentDisposition::attachment(filename))
        .streaming(ReaderStream::new(r)))
}

#[get("search")]
async fn search(query_map: Query<HashMap<String, String>>) -> Result<impl Responder> {
    if let Some(query) = query_map.get("q") {
        let filename_only = query_map
            .get("filename_only")
            .map(|s| s == "true")
            .unwrap_or(false);
        let results = IndexerData::search(query, filename_only).await?;
        Ok(HttpResponse::Ok().json(json!(results)))
    } else {
        Ok(HttpResponse::BadRequest().json(json!({
            "error": "Search query is required"
        })))
    }
}
#[post("refresh-index")]
pub async fn refresh_index() -> Result<impl Responder> {
    if let Err(e) = indexer_data::index_all_files().await {
        error!("Error starting file indexer: {}", e);
    }
    Ok(HttpResponse::Ok().finish())
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

    let mut file = match File::create(&path).await {
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
async fn copy_filesystem_entry(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    // Extract source paths
    let source_paths = body
        .get("entries")
        .and_then(|entries| entries.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|e| e.as_str().map(PathBuf::from))
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| anyhow::anyhow!("Invalid entries array"))?;

    // Extract destination path
    let dest_path = PathBuf::from(
        body.get("path")
            .and_then(|path| path.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid destination path"))?,
    );

    // Verify source paths exist
    for source_path in &source_paths {
        if !source_path.exists() {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": format!("Source path does not exist: {}", source_path.display())
            })));
        }
    }

    // Copy each source to destination
    for source_path in source_paths {
        let dest = dest_path.join(source_path.file_name().unwrap_or_default());

        if source_path.is_dir() {
            // Create a copy function for recursive directory copy
            fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
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

            if let Err(e) = copy_dir_all(&source_path, &dest) {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to copy directory: {}", e)
                })));
            }
        } else {
            // Copy file
            if let Err(e) = std::fs::copy(&source_path, &dest) {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to copy file: {}", e)
                })));
            }
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Entries copied successfully"
    })))
}

#[post("/move")]
async fn move_filesystem_entry(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    // Extract source paths
    let source_paths = body
        .get("entries")
        .and_then(|entries| entries.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|e| e.as_str().map(PathBuf::from))
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| anyhow::anyhow!("Invalid entries array"))?;

    // Extract destination path
    let dest_path = PathBuf::from(
        body.get("path")
            .and_then(|path| path.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid destination path"))?,
    );

    // Verify source paths exist
    for source_path in &source_paths {
        if !source_path.exists() {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": format!("Source path does not exist: {}", source_path.display())
            })));
        }
    }

    // Move each source to destination
    for source_path in source_paths {
        let dest = dest_path.join(source_path.file_name().unwrap_or_default());

        // Move/rename is the same operation in fs terms
        if let Err(e) = std::fs::rename(&source_path, &dest) {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to move entry: {}", e)
            })));
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Entries moved successfully"
    })))
}
#[delete("/")]
async fn delete_filesystem_entry(request: HttpRequest) -> Result<impl Responder> {
    let paths: Vec<PathBuf> = match request.headers().get("X-Filesystem-Paths") {
        Some(header) => match header.to_str() {
            Ok(path_str) => {
                serde_json::from_str::<Vec<PathBuf>>(path_str).map_err(anyhow::Error::msg)?
            }
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

    for path in paths {
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
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Entry deleted successfully"
    })))
}

#[post("/new")]
async fn new_filesystem_entry(request: HttpRequest) -> Result<impl Responder> {
    let file_path = request
        .headers()
        .get("X-Filesystem-Path")
        .and_then(|h| h.to_str().ok())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("X-Filesystem-Path header is missing"))?;

    let is_directory = request
        .headers()
        .get("X-Is-Directory")
        .and_then(|h| h.to_str().ok())
        .map(|s| s == "true")
        .unwrap_or(false);

    if is_directory {
        fs::create_dir_all(file_path)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
    } else {
        File::create(file_path)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
    }

    Ok(HttpResponse::Ok().finish())
}

#[get("/indexer/stats")]
async fn get_indexer_stats() -> Result<impl Responder> {
    match IndexerData::get_stats().await {
        Ok((count, total_size, avg_size)) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "stats": {
                "fileCount": count,
                "totalSize": total_size,
                "averageSize": avg_size,
                "humanReadableTotalSize": format_size(total_size),
                "humanReadableAverageSize": format_size(avg_size)
            }
        }))),
        Err(e) => {
            error!("Error getting indexer stats: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Failed to get indexer statistics: {}", e)
            })))
        }
    }
}

#[post("/archive")]
async fn archive_paths(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let filenames = body
        .get("entries")
        .and_then(|entries| entries.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|e| e.as_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| anyhow::anyhow!("Invalid entries array"))?;
    let cwd = PathBuf::from(
        body.get("cwd")
            .and_then(|cwd| cwd.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid cwd"))?,
    );
    let archive_file_name = body
        .get("filename")
        .and_then(|filename| filename.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
    let tracker_id = body
        .get("tracker_id")
        .and_then(|filename| filename.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing tracker id"))?;
    let pathed_entries = filenames
        .iter()
        .map(|filename| cwd.join(filename))
        .collect::<Vec<_>>();
    let archive_path = cwd.join(archive_file_name);

    let trackers = get_archive_trackers().lock().await;
    if let Some(tracker) = trackers.get(tracker_id) {
        archive_wrapper::archive(archive_path, pathed_entries, tracker)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
    } else {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid tracker id"
        })));
    }

    Ok(HttpResponse::Ok().finish())
}

#[get("/archive/status/{tracker_id}")]
async fn get_archive_status(tracker_id: web::Path<String>) -> impl Responder {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Store the sender in our tracker
    {
        let mut trackers = get_archive_trackers().lock().await;
        trackers.insert(tracker_id.to_string(), tx);
    }

    Sse::from_infallible_receiver(rx).with_keep_alive(Duration::from_secs(3))
}

// Helper function to format file sizes in a human-readable format
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if size < KB {
        format!("{} B", size)
    } else if size < MB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size < TB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else {
        format!("{:.2} TB", size as f64 / TB as f64)
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/filesystem")
            .wrap(Authentication::new())
            .service(get_filesystem_entries)
            .service(archive_paths)
            .service(get_archive_status)
            .service(download)
            .service(search)
            .service(upload)
            .service(upload_progress)
            .service(copy_filesystem_entry)
            .service(move_filesystem_entry)
            .service(delete_filesystem_entry)
            .service(new_filesystem_entry)
            .service(get_indexer_stats),
    );
}
