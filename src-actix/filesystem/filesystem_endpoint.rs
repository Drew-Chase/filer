use crate::auth::auth_middleware::Authentication;
use crate::filesystem::channel_writer::ChannelWriter;
use crate::filesystem::download_parameters::DownloadParameters;
use crate::filesystem::filesystem_data::{FilesystemData, FilesystemEntry};
use crate::helpers::http_error::Result;
use actix_web::web::{Bytes, Query};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use actix_web_lab::__reexports::futures_util::StreamExt;
use actix_web_lab::sse::{Data, Event, Sse};
use async_stream::stream;
use futures::Stream;
use serde_json::json;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use sysinfo::Disks;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{CompressionMethod, ZipWriter};

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
async fn download(query: Query<DownloadParameters>) -> Result<impl Responder> {
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
        uuid::Uuid::new_v4().to_string()
    };

    // If there is only one entry, and it's a file,
    // then stream the individual file to the client.
    if is_single_entry && !is_single_entry_directory {
        let data_stream: Pin<Box<dyn Stream<Item = Result<Bytes>>>> = Box::pin(stream! {
        let buffer_size = 64 * 1024; // 64KB buffer
        let mut read_buffer = vec![0; buffer_size];
            let mut reader = std::fs::File::open(items[0].clone()).map_err(|e| anyhow::anyhow!(e))?;
            loop {
                let bytes_read = reader.read(&mut read_buffer).map_err(|_| anyhow::Error::msg("Failed to read file bytes"))?;
                if bytes_read == 0 {
                    break;
                }
                yield Result::<Bytes>::Ok(Bytes::copy_from_slice(&read_buffer[..bytes_read]));
            }
            return;

        });
        return Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .insert_header((
                "Content-Disposition",
                format!(r#"attachment; filename="{}""#, filename),
            ))
            .streaming(data_stream));
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel::<std::result::Result<Bytes, io::Error>>(8);
    let cwd = query.cwd.clone();
    let items = items.clone();

    tokio::task::spawn_blocking(move || {
        let mut writer = ChannelWriter::new(tx);
        let mut zip = ZipWriter::new(&mut writer);
        let options: SimpleFileOptions =
            FileOptions::default().compression_method(CompressionMethod::Deflated);

        // Collect all files paths to put in the zip
        let items_to_write = if is_single_entry_directory {
            std::fs::read_dir(items[0].clone())
                .unwrap()
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .collect::<Vec<PathBuf>>()
        } else {
            items
        };

        let mut read_buffer = vec![0; 64 * 1024];

        for item in items_to_write {
            if let Some(filename) = item.file_name() {
                let filename = filename.to_string_lossy().to_owned();
                if item.is_dir() {
                    zip.add_directory(filename.clone(), options).unwrap();
                    let sub_paths = walkdir::WalkDir::new(item.clone());
                    for entry in sub_paths {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.is_dir() {
                                continue;
                            }
                            let path_str = path.to_string_lossy().replace("\\", "/");
                            let relative_path = path_str.replace(&format!("{}/", cwd), "");
                            zip.start_file(relative_path, options).unwrap();
                            let mut file = std::fs::File::open(&path).unwrap();
                            loop {
                                let bytes_read = file.read(&mut read_buffer).unwrap();
                                if bytes_read == 0 {
                                    break;
                                }
                                zip.write_all(&read_buffer[..bytes_read]).unwrap();
                            }
                        }
                    }
                } else {
                    zip.start_file(filename, options).unwrap();
                    let mut file = std::fs::File::open(item.clone()).unwrap();
                    loop {
                        let bytes_read = file.read(&mut read_buffer).unwrap();
                        if bytes_read == 0 {
                            break;
                        }
                        zip.write_all(&read_buffer[..bytes_read]).unwrap();
                    }
                }
            }
        }

        zip.finish().unwrap();
        writer.flush().unwrap();
    });

    let data_stream: Pin<Box<dyn Stream<Item = Result<Bytes>>>> = Box::pin(stream! {
            while let Some(chunk) = rx.recv().await {
                if let Ok(bytes) = chunk{
                  yield Result::<Bytes>::Ok(Bytes::copy_from_slice(&bytes));
                }
            }
    });

    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .insert_header((
            "Content-Disposition",
            format!(r#"attachment; filename="{}""#, filename),
        ))
        .streaming(data_stream))
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
