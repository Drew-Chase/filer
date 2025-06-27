use crate::auth::auth_middleware::Authentication;
use crate::helpers::http_error::{Error, Result};
use crate::io::fs::archive_wrapper;
use crate::io::fs::download_parameters::DownloadParameters;
use crate::io::fs::filesystem_data::{FilesystemData, FilesystemEntry};
use crate::io::fs::indexer::indexer_data;
use crate::io::fs::indexer::indexer_data::IndexerData;
use crate::io::fs::normalize_path::NormalizePath;
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
use std::sync::atomic::{AtomicBool, Ordering};
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
static ARCHIVE_CANCEL_FLAGS: OnceLock<ArchiveCancelFlags> = OnceLock::new();
static UPLOAD_CANCEL_FLAGS: OnceLock<UploadCancelFlags> = OnceLock::new();

type FileProcessTracker = Arc<Mutex<HashMap<String, Sender<Event>>>>;
type ArchiveCancelFlags = Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>;
type UploadCancelFlags = Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>;

// Helper function to get or initialize the tracker
fn get_upload_trackers() -> &'static FileProcessTracker {
    UPLOAD_TRACKERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}
fn get_archive_trackers() -> &'static FileProcessTracker {
    ARCHIVE_TRACKERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}
fn get_archive_cancel_flags() -> &'static ArchiveCancelFlags {
    ARCHIVE_CANCEL_FLAGS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}
fn get_upload_cancel_flags() -> &'static UploadCancelFlags {
    UPLOAD_CANCEL_FLAGS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

#[get("/")]
async fn get_filesystem_entries(request: HttpRequest) -> Result<impl Responder> {
    let path = match request.headers().get("X-Filesystem-Path") {
        Some(header) => match header.to_str() {
            Ok(path_str) => path_str.to_os_path(),
            Err(_) => {
                return Err(Error::invalid_input(
                    "X-Filesystem-Path header is not a valid string",
                ));
            }
        },
        None => {
            return Err(Error::validation_error(
                "X-Filesystem-Path header is missing",
                Some("X-Filesystem-Path"),
            ));
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
            let path = PathBuf::from("/");
            // Continue to the normal path handling below
            let entries: FilesystemData = path.try_into()?;
            return Ok(HttpResponse::Ok().json(json!(entries)));
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
    let cwd = query.cwd.to_os_path();
    let items: Vec<PathBuf> = query
        .items
        .iter()
        .map(|item| format!("{}{}", query.cwd, item).to_os_path())
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

        let file = File::open(&filepath).await.map_err(|e| {
            Error::filesystem_error(
                format!("Failed to open file for download: {}", filepath.display()),
                Some(e),
                Some(filepath.clone()),
            )
        })?;
        let stream = ReaderStream::new(file);

        return Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .insert_header(ContentDisposition::attachment(filename))
            .streaming(stream));
    }

    // For directories or multiple files, create a zip archive
    let (w, r) = duplex(4096);
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
            Ok(path_str) => path_str.to_os_path(),
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

    // Create a cancellation flag for this upload
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut cancel_flags = get_upload_cancel_flags().lock().await;
        cancel_flags.insert(upload_id.clone(), cancel_flag.clone());
    }

    let mut file = match File::create(&path).await {
        Ok(file) => file,
        Err(_) => {
            // Clean up the cancellation flag
            let mut cancel_flags = get_upload_cancel_flags().lock().await;
            cancel_flags.remove(&upload_id);

            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create file"
            }));
        }
    };

    let mut total_bytes = 0u64;

    // Process the upload
    while let Some(chunk) = payload.next().await {
        // Check if upload was cancelled
        if cancel_flag.load(Ordering::Relaxed) {
            info!("Upload operation with ID {} cancelled by user", upload_id);

            // Send cancellation event
            if let Some(sender) = &progress_sender {
                let _ = sender
                    .send(Event::from(Data::new(
                        json!({
                            "status": "cancelled",
                            "bytesUploaded": total_bytes
                        })
                        .to_string(),
                    )))
                    .await;
            }

            // Clean up
            let mut cancel_flags = get_upload_cancel_flags().lock().await;
            cancel_flags.remove(&upload_id);

            // Close and delete the partial file
            file.shutdown().await.ok();
            fs::remove_file(&path).await.ok();

            return HttpResponse::Ok().json(json!({
                "status": "cancelled",
                "message": "Upload cancelled by user"
            }));
        }

        match chunk {
            Ok(bytes) => {
                if file.write_all(&bytes).await.is_err() {
                    // Clean up the cancellation flag
                    let mut cancel_flags = get_upload_cancel_flags().lock().await;
                    cancel_flags.remove(&upload_id);

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
                // Clean up the cancellation flag
                let mut cancel_flags = get_upload_cancel_flags().lock().await;
                cancel_flags.remove(&upload_id);

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

    // Clean up the cancellation flag
    let mut cancel_flags = get_upload_cancel_flags().lock().await;
    cancel_flags.remove(&upload_id);

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
                .filter_map(|e| e.as_str().map(|i| i.to_os_path()))
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| anyhow::anyhow!("Invalid entries array"))?;

    // Extract destination path
    let dest_path = body
        .get("path")
        .and_then(|path| path.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid destination path"))?
        .to_os_path();

    // Verify source paths exist
    for source_path in &source_paths {
        if !source_path.exists() {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": format!("Source path does not exist: {}", source_path.display())
            })));
        }
    }

    // Copy each source to the destination
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
                .filter_map(|e| e.as_str().map(|i| i.to_os_path()))
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| anyhow::anyhow!("Invalid entries array"))?;

    // Extract destination path
    let dest_path = body
        .get("path")
        .and_then(|path| path.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid destination path"))?
        .to_os_path();

    // Move each source to a destination
    for source_path in source_paths {
        if !source_path.exists() {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": format!("Source path does not exist: {}", source_path.display())
            })));
        }
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
#[post("/rename")]
async fn rename_filesystem_entry(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    // Extract destination path
    let source_path = body
        .get("source")
        .and_then(|path| path.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid source path"))?
        .to_os_path();
    let dest_path = body
        .get("destination")
        .and_then(|path| path.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid destination path"))?
        .to_os_path();

    if !source_path.exists() {
        return Ok(HttpResponse::NotFound().json(json!({
            "error": format!("Source path does not exist: {}", source_path.display())
        })));
    }

    if let Err(e) = std::fs::rename(&source_path, &dest_path) {
        return Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to move entry: {}", e)
        })));
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Entries moved successfully"
    })))
}
#[delete("/")]
async fn delete_filesystem_entry(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let paths = match body.get("paths") {
        Some(paths) => match paths.as_array() {
            Some(array) => array
                .iter()
                .filter_map(|p| p.as_str().map(|i| i.to_os_path()))
                .collect::<Vec<PathBuf>>(),
            None => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "error": "paths must be an array"
                })));
            }
        },
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "paths field is required"
            })));
        }
    };

    for path in paths {
        // Verify a path exists
        if !path.exists() {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Path does not exist",
                "path": path.to_string_lossy().into_owned()
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
async fn new_filesystem_entry(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let file_path = body
        .get("path")
        .and_then(|p| p.as_str())
        .map(|i| i.to_os_path())
        .ok_or_else(|| Error::validation_error("path field is missing", Some("path")))?;

    let is_directory = body
        .get("is_directory")
        .and_then(|d| d.as_bool())
        .unwrap_or(false);

    if is_directory {
        fs::create_dir_all(&file_path).await.map_err(|e| {
            Error::filesystem_error(
                format!("Failed to create directory: {}", file_path.display()),
                Some(e),
                Some(file_path.clone()),
            )
        })?;
    } else {
        File::create(&file_path).await.map_err(|e| {
            Error::filesystem_error(
                format!("Failed to create file: {}", file_path.display()),
                Some(e),
                Some(file_path.clone()),
            )
        })?;
    }

    Ok(HttpResponse::Ok().finish())
}

#[get("/indexer/stats")]
async fn get_indexer_stats() -> Result<impl Responder> {
    let (count, total_size, avg_size) = IndexerData::get_stats().await.map_err(|e| {
        error!("Error getting indexer stats: {}", e);
        Error::database_error(
            format!("Failed to get indexer statistics: {}", e),
            Some(anyhow::anyhow!(e)),
        )
    })?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "stats": {
            "fileCount": count,
            "totalSize": total_size,
            "averageSize": avg_size,
            "humanReadableTotalSize": format_size(total_size),
            "humanReadableAverageSize": format_size(avg_size)
        }
    })))
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
        .ok_or_else(|| Error::validation_error("Invalid entries array", Some("entries")))?;
    let cwd = body
        .get("cwd")
        .and_then(|cwd| cwd.as_str())
        .ok_or_else(|| {
            Error::validation_error("Current working directory is required", Some("cwd"))
        })?
        .to_os_path();
    let archive_file_name = body
        .get("filename")
        .and_then(|filename| filename.as_str())
        .ok_or_else(|| Error::validation_error("Archive filename is required", Some("filename")))?;
    let tracker_id = body
        .get("tracker_id")
        .and_then(|filename| filename.as_str())
        .ok_or_else(|| Error::validation_error("Tracker ID is required", Some("tracker_id")))?;
    let absolute_file_paths = filenames
        .iter()
        .map(|filename| cwd.join(filename))
        .collect::<Vec<_>>();
    let archive_path = cwd.join(archive_file_name);

    let trackers = get_archive_trackers().lock().await;
    if let Some(tracker) = trackers.get(tracker_id) {
        // Create a new cancellation flag for this operation
        let cancel_flag = Arc::new(AtomicBool::new(false));

        // Store the cancellation flag
        {
            let mut cancel_flags = get_archive_cancel_flags().lock().await;
            cancel_flags.insert(tracker_id.to_string(), cancel_flag.clone());
        }

        // Run the archive operation with the cancellation flag
        archive_wrapper::archive(
            archive_path.clone(),
            absolute_file_paths,
            tracker,
            &cancel_flag,
        )
        .await
        .map_err(|_| {
            Error::filesystem_error(
                format!("Failed to create archive: {}", archive_path.display()),
                None,
                Some(archive_path.clone()),
            )
        })?;

        // Clean up the cancellation flag
        {
            let mut cancel_flags = get_archive_cancel_flags().lock().await;
            cancel_flags.remove(tracker_id);
        }
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

#[post("/archive/cancel/{tracker_id}")]
async fn cancel_archive(tracker_id: web::Path<String>) -> Result<impl Responder> {
    let tracker_id = tracker_id.into_inner();

    // Get the cancellation flag for this tracker
    let cancel_flags = get_archive_cancel_flags().lock().await;

    if let Some(flag) = cancel_flags.get(&tracker_id) {
        // Set the flag to true to signal cancellation
        flag.store(true, Ordering::Relaxed);
        info!("Archive operation with ID {} cancelled by user", tracker_id);

        Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Archive operation cancelled"
        })))
    } else {
        // If the tracker doesn't exist, it might have already completed or never existed
        warn!(
            "Attempted to cancel non-existent archive operation with ID {}",
            tracker_id
        );

        Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Archive operation not found or already completed"
        })))
    }
}

#[post("/upload/cancel/{upload_id}")]
async fn cancel_upload(upload_id: web::Path<String>) -> Result<impl Responder> {
    let upload_id = upload_id.into_inner();

    // Get the cancellation flag for this upload
    let cancel_flags = get_upload_cancel_flags().lock().await;

    if let Some(flag) = cancel_flags.get(&upload_id) {
        // Set the flag to true to signal cancellation
        flag.store(true, Ordering::Relaxed);
        info!("Upload operation with ID {} cancelled by user", upload_id);

        Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Upload operation cancelled"
        })))
    } else {
        // If the tracker doesn't exist, it might have already completed or never existed
        warn!(
            "Attempted to cancel non-existent upload operation with ID {}",
            upload_id
        );

        Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Upload operation not found or already completed"
        })))
    }
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
            .service(get_filesystem_entries)
            .service(
                web::scope("")
                    .wrap(Authentication::new())
                    .service(archive_paths)
                    .service(get_archive_status)
                    .service(cancel_archive)
                    .service(download)
                    .service(search)
                    .service(upload)
                    .service(upload_progress)
                    .service(cancel_upload)
                    .service(copy_filesystem_entry)
                    .service(move_filesystem_entry)
                    .service(rename_filesystem_entry)
                    .service(delete_filesystem_entry)
                    .service(new_filesystem_entry)
                    .service(get_indexer_stats),
            ),
    );
}
