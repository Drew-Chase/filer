use crate::configuration::configuration_data::Configuration;
use crate::helpers::db::create_pool;
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sqlx::{FromRow, SqlitePool};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromRow)]
pub struct IndexerData {
    pub path: String,
    pub filename: String,
    pub size: u64,
    pub mtime: u64,
    pub ctime: u64,
}

// Global watcher state
static mut FILE_WATCHER: Option<Arc<Mutex<FileWatcherState>>> = None;

struct FileWatcherState {
    watcher: RecommendedWatcher,
}

pub async fn index_all_files() -> Result<()> {
    info!("Starting file indexing...");
    let start_time = std::time::Instant::now();

    let config = Configuration::get();
    let is_ignored_path_whitelist = config.filter_mode_whitelist;
    let ignored_paths = &config.filter;

    // Create a database connection pool
    let pool = create_pool().await?;

    // Get all disk mount points
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut indexed_count = 0;
    let mut error_count = 0;

    // Process files in batches for better performance
    let batch_size = 1000;
    let mut batch = Vec::with_capacity(batch_size);

    for disk in &disks {
        let mount_point = disk.mount_point();
        info!("Indexing disk: {:?}", mount_point);

        // Use WalkDir for efficient directory traversal
        for entry in WalkDir::new(mount_point).follow_links(false).into_iter().filter_map(|e| e.ok()) {
            // Skip directories, only index files
            if entry.file_type().is_file() {
                let path = entry.path();
                let path_str = path.to_string_lossy().replace('\\', "/");

                // Check if the path matches any ignored patterns
                let matches_pattern = ignored_paths.iter().any(|ignored| {
                    let pattern = glob::Pattern::new(ignored).unwrap_or_default();
                    pattern.matches(&path_str)
                });

                // Check if the file is hidden and should be excluded
                let is_hidden = path.file_name().and_then(|name| name.to_str()).map(|name| name.starts_with(".")).unwrap_or(false);

                if ((!is_ignored_path_whitelist && matches_pattern) || (is_ignored_path_whitelist && !matches_pattern))
                    || (config.exclude_hidden_files && is_hidden)
                {
                    continue;
                }

                match IndexerData::from_path(path) {
                    Ok(data) => {
                        batch.push(data);

                        // When the batch is full, insert into the database
                        if batch.len() >= batch_size {
                            if let Err(e) = insert_batch(&batch, &pool).await {
                                error!("Error inserting batch: {}", e);
                                error_count += batch.len();
                            } else {
                                indexed_count += batch.len();
                            }
                            batch.clear();
                        }
                    }
                    Err(e) => {
                        error!("Error processing file {:?}: {}", path, e);
                        error_count += 1;
                    }
                }
            }
        }
    }

    // Insert any remaining files in the batch
    if !batch.is_empty() {
        if let Err(e) = insert_batch(&batch, &pool).await {
            error!("Error inserting final batch: {}", e);
            error_count += batch.len();
        } else {
            indexed_count += batch.len();
        }
    }

    let elapsed = start_time.elapsed();
    let hours = elapsed.as_secs() / 3600;
    let minutes = (elapsed.as_secs() % 3600) / 60;
    let seconds = elapsed.as_secs() % 60;
    info!("Indexing completed in {}h {}m {}s. Indexed {} files, {} errors.", hours, minutes, seconds, indexed_count, error_count);

    Ok(())
}

async fn insert_batch(batch: &[IndexerData], pool: &SqlitePool) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    // Begin transaction for better performance
    let mut tx = pool.begin().await?;

    // Use a single query with multiple value sets for better performance
    let mut query = String::from("INSERT OR REPLACE INTO indexes (path, filename, mtime, ctime, size) VALUES ");

    // Create placeholders for all items in the batch
    let placeholders: Vec<String> = (0..batch.len())
        .map(|i| {
            format!(
                "(${}, ${}, ${}, ${}, ${})",
                i * 5 + 1, // Fixed: 5 columns, not 4
                i * 5 + 2,
                i * 5 + 3,
                i * 5 + 4,
                i * 5 + 5
            )
        })
        .collect();

    query.push_str(&placeholders.join(", "));

    // Prepare the query with all parameters
    let mut q = sqlx::query(&query);

    // Bind all parameters
    for data in batch {
        q = q.bind(&data.path).bind(&data.filename).bind(data.mtime as i64).bind(data.ctime as i64).bind(data.size as i64);
    }

    // Execute the query with all parameters
    q.execute(&mut *tx).await?;

    // Commit transaction
    tx.commit().await?;
    Ok(())
}

pub async fn start_file_watcher() -> Result<()> {
    info!("Starting file watcher...");

    let notify_config = Config::default().with_poll_interval(Duration::from_secs(2)).with_compare_contents(false);

    let (tx, rx) = std::sync::mpsc::channel();
    let watcher = RecommendedWatcher::new(tx, notify_config)?;

    let watcher_state = Arc::new(Mutex::new(FileWatcherState { watcher }));

    unsafe {
        FILE_WATCHER = Some(watcher_state.clone());
    }

    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut state = watcher_state.lock().await;

    for disk in &disks {
        let mount_point = disk.mount_point();
        info!("Watching directory: {:?}", mount_point);
        if let Err(e) = state.watcher.watch(mount_point, RecursiveMode::Recursive) {
            warn!("Error watching {:?}: {}", mount_point, e);
        }
    }

    tokio::spawn(async move {
        loop {
            match rx.recv() {
                Ok(event) => {
                    if let Err(e) = process_file_event(event).await {
                        error!("Error processing file event: {}", e);
                    }
                }
                Err(e) => {
                    error!("File watcher error: {}", e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    info!("File watcher started successfully");
    Ok(())
}

async fn process_file_event(event: Result<Event, notify::Error>) -> Result<()> {
    let config = Configuration::get();
    let is_ignored_path_whitelist = config.filter_mode_whitelist;
    let ignored_paths = &config.filter;

    let event = event?;

    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {
            for path in event.paths {
                // Check if the path matches any ignored patterns
                let path_str = path.to_string_lossy().replace('\\', "/");
                let matches_pattern = ignored_paths.iter().any(|ignored| {
                    let pattern = glob::Pattern::new(ignored).unwrap_or_default();
                    pattern.matches(&path_str)
                });

                // Check if the file is hidden and should be excluded
                let is_hidden = path.file_name().and_then(|name| name.to_str()).map(|name| name.starts_with(".")).unwrap_or(false);

                if ((!is_ignored_path_whitelist && matches_pattern) || (is_ignored_path_whitelist && !matches_pattern))
                    || (config.exclude_hidden_files && is_hidden)
                {
                    continue;
                }

                if path.is_file() {
                    match IndexerData::from_path(&path) {
                        Ok(data) => {
                            debug!("Updating index for modified file: {}", data.path);

                            if let Ok(Some(_)) = IndexerData::get_by_path(&data.path).await {
                                data.update().await?;
                            } else {
                                data.insert().await?;
                            }
                        }
                        Err(e) => {
                            error!("Error processing modified file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        EventKind::Remove(_) => {
            for path in event.paths {
                // Check if the path matches any ignored patterns
                let path_str = path.to_string_lossy().replace('\\', "/");
                let matches_pattern = ignored_paths.iter().any(|ignored| {
                    let pattern = glob::Pattern::new(ignored).unwrap_or_default();
                    pattern.matches(&path_str)
                });

                // Check if the file is hidden and should be excluded
                let is_hidden = path.file_name().and_then(|name| name.to_str()).map(|name| name.starts_with(".")).unwrap_or(false);

                if ((!is_ignored_path_whitelist && matches_pattern) || (is_ignored_path_whitelist && !matches_pattern))
                    || (config.exclude_hidden_files && is_hidden)
                {
                    continue;
                }

                let path_str = path.to_string_lossy().to_string().replace('\\', "/");
                debug!("Removing index for deleted file: {}", path_str);

                if let Err(e) = IndexerData::delete(&path_str).await {
                    error!("Error removing index for {:?}: {}", path, e);
                }
            }
        }

        _ => {}
    }

    Ok(())
}

impl IndexerData {
    pub fn from_path(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path).context("Failed to get file metadata")?;

        // Get file size
        let size = metadata.len();
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        // Get modification time
        let mtime = metadata.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);

        // Get creation time
        let ctime = metadata.created().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);

        Ok(Self { path: path.to_string_lossy().to_string().replace('\\', "/"), filename, size, mtime, ctime })
    }

    // Utility method to check if a file has been modified since it was last indexed
    pub async fn is_modified(&self) -> Result<bool> {
        let path = Path::new(&self.path);
        if !path.exists() {
            return Ok(true); // Consider non-existent files as modified
        }

        let current = Self::from_path(path)?;
        Ok(current.size != self.size || current.mtime != self.mtime)
    }

    // Utility method to get all indexed files
    pub async fn get_all() -> Result<Vec<Self>> {
        let pool = create_pool().await?;
        let result = sqlx::query_as::<_, IndexerData>(r#"select * from indexes"#).fetch_all(&pool).await?;
        Ok(result)
    }

    // Utility method to check database statistics
    pub async fn get_stats() -> Result<(u64, u64, u64)> {
        let pool = create_pool().await?;

        // Get total count
        let count: (i64,) = sqlx::query_as(r#"select count(*) from indexes"#).fetch_one(&pool).await?;

        // Get total size
        let size: (i64,) = sqlx::query_as(r#"select sum(size) from indexes"#).fetch_one(&pool).await?;

        // Get average size
        let avg_size: (i64,) = sqlx::query_as(r#"select avg(size) from indexes"#).fetch_one(&pool).await?;

        Ok((count.0 as u64, size.0 as u64, avg_size.0 as u64))
    }
}
