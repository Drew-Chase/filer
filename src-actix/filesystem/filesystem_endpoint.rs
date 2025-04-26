use crate::auth::auth_middleware::Authentication;
use crate::filesystem::filesystem_data::{FilesystemData, FilesystemEntry};
use crate::helpers::http_error::{Error, Result};
use actix_web::web::{Bytes, Query};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_web_lab::__reexports::futures_util::StreamExt;
use actix_web_lab::sse::{Data, Event, Sse};
use async_stream::stream;
use futures::Stream;
use serde_json::json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use sysinfo::Disks;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
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
        let data_stream: Pin<Box<dyn Stream<Item = Result<Bytes>>>> = Box::pin(stream! {
            let mut buffer = Vec::new();
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated);

            let dir = walkdir::WalkDir::new(entry.path);
            for entry_result in dir {
                let entry = entry_result.map_err(|_| Error::InternalError(anyhow::Error::msg("Failed to read directory")))?;
                let path = entry.path();
                if path.is_file() {
                    let relative_path = path.strip_prefix(entry.path())
                        .map_err(|_| Error::InternalError(anyhow::Error::msg("Failed to get relative path")))?;
                    zip.start_file(relative_path.to_string_lossy().into_owned(), options)
                        .map_err(|_| Error::InternalError(anyhow::Error::msg("Failed to create zip file.")))?;

                    // Read the file in chunks and write to zip
                    let mut file = std::fs::File::open(path)
                        .map_err(|_| Error::InternalError(anyhow::Error::msg("Failed to open file.")))?;
                    buffer.clear();
                    buffer.resize(64 * 1024, 0); // 64KB buffer

                    loop {
                        let bytes_read = file.read(&mut buffer)
                            .map_err(|_| Error::InternalError(anyhow::Error::msg("Failed to read file")))?;
                        if bytes_read == 0 { break; }

                        zip.write_all(&buffer[..bytes_read])
                            .map_err(|_| Error::InternalError(anyhow::Error::msg("Failed to write to zip")))?;
                    }
                }
            }

            // Get the final zip data
            let cursor = zip.finish().map_err(|_| Error::InternalError(anyhow::Error::msg("Failed to finish zip file")))?;
            let final_data = cursor.into_inner();

            // Stream the final data in chunks
            for chunk in final_data.chunks(64 * 1024) {
                yield Ok(Bytes::from(chunk.to_vec()));
            }
        });
        Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .insert_header((
                "Content-Disposition",
                format!(r#"attachment; filename="{}""#, entry.filename),
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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/filesystem")
            .wrap(Authentication::new())
            .service(get_filesystem_entries)
            .service(download)
            .service(search)
            .service(upload)
            .service(upload_progress),
    );
}
