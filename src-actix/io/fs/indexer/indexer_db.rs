use crate::io::fs::indexer::indexer_data::IndexerData;
use log::LevelFilter;
use sqlx::sqlite::SqliteSynchronous::Normal;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{ConnectOptions, Executor, Row, SqlitePool};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::helpers::db::create_pool;

pub async fn initialize() -> anyhow::Result<()> {
    let pool = create_pool().await?;

    // Create the main indexes table if it doesn't exist
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS indexes
(
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    path     TEXT UNIQUE,
    filename TEXT    NOT NULL,
    mtime    INTEGER NOT NULL,
    ctime    INTEGER NOT NULL,
    size     INTEGER NOT NULL
);
        "#,
    )
    .await?;

    // Create the trigram table if it doesn't exist
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS path_trigrams
(
    id       INTEGER PRIMARY KEY,
    path_id  INTEGER NOT NULL,
    trigram  TEXT NOT NULL,
    FOREIGN KEY (path_id) REFERENCES indexes(id)
);
        "#,
    )
    .await?;

    // Create an index on trigrams for faster searching
    pool.execute(
        r#"CREATE INDEX IF NOT EXISTS idx_trigram ON path_trigrams(trigram);"#,
    )
    .await?;

    // Check if the trigram table is empty
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM path_trigrams")
        .fetch_one(&pool)
        .await?;

    // If the trigram table is empty but we have data in the indexes table,
    // rebuild the trigram index
    if count == 0 {
        let index_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM indexes")
            .fetch_one(&pool)
            .await?;

        if index_count > 0 {
            log::info!("Rebuilding trigram index for {} existing records...", index_count);
            rebuild_trigram_index().await?;
            log::info!("Trigram index rebuilt successfully.");
        }
    }

    Ok(())
}
impl IndexerData {
    pub async fn insert_with_pool(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        // Start a transaction
        let mut tx = pool.begin().await?;

        // Insert into the main indexes table
        let result = sqlx::query(
            r#"INSERT INTO indexes (path, filename, mtime, ctime, size) VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(&self.path)
        .bind(&self.filename)
        .bind(self.mtime as i64)
        .bind(self.ctime as i64)
        .bind(self.size as i64)
        .execute(&mut *tx)
        .await?;

        // Get the last inserted ID
        let id = result.last_insert_rowid();

        // Generate trigrams for path and filename (for better search)
        let path_trigrams = generate_trigrams(&self.path);
        let filename_trigrams = generate_trigrams(&self.filename);

        // Combine trigrams, remove duplicates
        let mut all_trigrams = path_trigrams;
        for trigram in filename_trigrams {
            if !all_trigrams.contains(&trigram) {
                all_trigrams.push(trigram);
            }
        }

        // Insert trigrams
        for trigram in all_trigrams {
            sqlx::query(
                r#"INSERT INTO path_trigrams (path_id, trigram) VALUES (?, ?)"#
            )
            .bind(id)
            .bind(&trigram)
            .execute(&mut *tx)
            .await?;
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }

    pub async fn insert(&self) -> anyhow::Result<()> {
        let pool = create_pool().await?;
        self.insert_with_pool(&pool).await
    }

    pub async fn update_with_pool(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        // Start a transaction
        let mut tx = pool.begin().await?;

        // Update the main record
        sqlx::query(r#"UPDATE indexes SET mtime = ?, ctime = ?, size = ? WHERE path = ?"#)
            .bind(self.mtime as i64)
            .bind(self.ctime as i64)
            .bind(self.size as i64)
            .bind(&self.path)
            .execute(&mut *tx)
            .await?;

        // Get the ID for this path
        let row = sqlx::query(r#"SELECT id FROM indexes WHERE path = ?"#)
            .bind(&self.path)
            .fetch_one(&mut *tx)
            .await?;

        let id: i64 = row.get(0);

        // Delete existing trigrams
        sqlx::query(r#"DELETE FROM path_trigrams WHERE path_id = ?"#)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Generate trigrams for path and filename (for better search)
        let path_trigrams = generate_trigrams(&self.path);
        let filename_trigrams = generate_trigrams(&self.filename);

        // Combine trigrams, remove duplicates
        let mut all_trigrams = path_trigrams;
        for trigram in filename_trigrams {
            if !all_trigrams.contains(&trigram) {
                all_trigrams.push(trigram);
            }
        }

        // Insert trigrams
        for trigram in all_trigrams {
            sqlx::query(
                r#"INSERT INTO path_trigrams (path_id, trigram) VALUES (?, ?)"#
            )
            .bind(id)
            .bind(&trigram)
            .execute(&mut *tx)
            .await?;
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }

    pub async fn update(&self) -> anyhow::Result<()> {
        let pool = create_pool().await?;
        self.update_with_pool(&pool).await
    }

    pub async fn delete_with_pool(path: &str, pool: &SqlitePool) -> anyhow::Result<()> {
        // Start a transaction
        let mut tx = pool.begin().await?;

        // Get the ID for this path
        let row_opt = sqlx::query(r#"SELECT id FROM indexes WHERE path = ?"#)
            .bind(path)
            .fetch_optional(&mut *tx)
            .await?;

        if let Some(row) = row_opt {
            let id: i64 = row.get(0);

            // Delete trigrams first (due to foreign key constraint)
            sqlx::query(r#"DELETE FROM path_trigrams WHERE path_id = ?"#)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        // Then delete the main record
        sqlx::query(r#"DELETE FROM indexes WHERE path = ?"#)
            .bind(path)
            .execute(&mut *tx)
            .await?;

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }

    pub async fn delete(path: &str) -> anyhow::Result<()> {
        let pool = create_pool().await?;
        Self::delete_with_pool(path, &pool).await
    }

    pub async fn get_by_path(path: &str) -> anyhow::Result<Option<Self>> {
        let pool = create_pool().await?;
        let result = sqlx::query_as::<_, IndexerData>(r#"select * from indexes where path = ?"#)
            .bind(path)
            .fetch_optional(&pool)
            .await?;
        Ok(result)
    }

    pub async fn does_table_exist() -> anyhow::Result<bool> {
        let pool = create_pool().await?;
        let result = sqlx::query(
            r#"SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'indexes'"#,
        )
        .fetch_one(&pool)
        .await?;

        Ok(result.get::<i32, _>(0) > 0)
    }

    pub async fn get_entries_in_directory(path: impl AsRef<Path>) -> anyhow::Result<Vec<Self>> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy().to_string();
        let path_str = path_str.replace('\\', "/");
        let pool = create_pool().await?;
        let query = format!("{}%", path_str);
        let result = sqlx::query_as::<_, IndexerData>(r#"select * from indexes where path like ?"#)
            .bind(&query)
            .fetch_all(&pool)
            .await?;
        let in_dir: Vec<IndexerData> = result
            .iter()
            .filter(|entry| {
                let entry_path = PathBuf::from(&entry.path);
                if let Ok(path) = entry_path.strip_prefix(path) {
                    !path.to_string_lossy().to_string().contains("/")
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Ok(in_dir)
    }

    pub async fn get_directory_size(path: impl AsRef<Path>) -> anyhow::Result<u64> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy().to_string();
        let path_str = path_str.replace('\\', "/");
        let pool = create_pool().await?;
        let query = format!("{}%", path_str);
        let result = sqlx::query_as::<_, IndexerData>(r#"select * from indexes where path like ?"#)
            .bind(&query)
            .fetch_all(&pool)
            .await?;

        let size: u64 = result.iter().map(|item| item.size).sum();

        Ok(size)
    }

    pub async fn search(query: impl AsRef<str>, filename_only: bool) -> anyhow::Result<Vec<Self>> {
        if query.as_ref().is_empty() {
            return Ok(Vec::new());
        }

        let pool = create_pool().await?;
        let search_term = query.as_ref();

        // Generate trigrams from the search term
        let search_trigrams = generate_trigrams(search_term);

        if search_trigrams.is_empty() {
            return Ok(Vec::new());
        }

        // First, try exact search for very short queries
        if search_term.len() <= 3 {
            let exact_pattern = format!("%{}%", search_term);

            let field = if filename_only { "filename" } else { "path" };
            let exact_results = sqlx::query_as::<_, IndexerData>(
                &format!(r#"SELECT * FROM indexes WHERE {} LIKE ? LIMIT 100"#, field)
            )
            .bind(&exact_pattern)
            .fetch_all(&pool)
            .await?;

            if !exact_results.is_empty() {
                return Ok(exact_results);
            }
        }

        // Create a query that finds paths with the most matching trigrams
        let sql_condition = if filename_only {
            // Join with indexes to filter by filename trigrams
            "JOIN indexes i ON pt.path_id = i.id"
        } else {
            // Join with indexes without additional filtering
            "JOIN indexes i ON pt.path_id = i.id"
        };

        // Create placeholders for the IN clause
        let placeholders = vec!["?"; search_trigrams.len()].join(",");

        let query_str = format!(
            r#"SELECT i.path, i.filename, i.mtime, i.ctime, i.size, COUNT(DISTINCT pt.trigram) as match_count
            FROM path_trigrams pt
            {}
            WHERE pt.trigram IN ({})
            GROUP BY pt.path_id
            ORDER BY match_count DESC, 
                     CASE WHEN i.{} LIKE ? THEN 1 ELSE 0 END DESC,
                     LENGTH(i.{})
            LIMIT 100"#,
            sql_condition, 
            placeholders,
            if filename_only { "filename" } else { "path" },
            if filename_only { "filename" } else { "path" }
        );

        // Build the query with all trigram parameters
        let mut query = sqlx::query(&query_str);

        // Bind all trigrams as parameters
        for trigram in &search_trigrams {
            query = query.bind(trigram);
        }

        // Bind the LIKE pattern for prioritizing direct matches
        query = query.bind(format!("%{}%", search_term));

        // Execute the query
        let rows = query.fetch_all(&pool).await?;

        // Convert the results to IndexerData structs
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            let indexer_data = IndexerData {
                path: row.get("path"),
                filename: row.get("filename"),
                mtime: row.get::<i64, _>("mtime") as u64,
                ctime: row.get::<i64, _>("ctime") as u64,
                size: row.get::<i64, _>("size") as u64,
            };
            results.push(indexer_data);
        }

        // If we didn't get many results from trigram search, fallback to LIKE
        if results.len() < 5 {
            let like_pattern = format!("%{}%", search_term);
            let field = if filename_only { "filename" } else { "path" };

            let like_results = sqlx::query_as::<_, IndexerData>(
                &format!(r#"SELECT * FROM indexes WHERE {} LIKE ? LIMIT 100"#, field)
            )
            .bind(&like_pattern)
            .fetch_all(&pool)
            .await?;

            // Combine results, removing duplicates
            for item in like_results {
                if !results.iter().any(|x| x.path == item.path) {
                    results.push(item);
                }
            }
        }

        Ok(results)
    }
}

// Function to generate trigrams from a string
fn generate_trigrams(text: &str) -> Vec<String> {
    let text = text.to_lowercase();
    let chars: Vec<char> = text.chars().collect();

    if chars.len() < 3 {
        return vec![text];
    }

    let mut trigrams = Vec::new();
    for i in 0..=(chars.len() - 3) {
        let trigram: String = chars[i..(i + 3)].iter().collect();
        trigrams.push(trigram);
    }

    trigrams
}

// Function to rebuild the trigram index for existing data
pub async fn rebuild_trigram_index() -> anyhow::Result<()> {
    let pool = create_pool().await?;

    // Clear existing trigrams
    sqlx::query("DELETE FROM path_trigrams").execute(&pool).await?;

    // Get all indexes data
    let rows = sqlx::query("SELECT id, path, filename FROM indexes").fetch_all(&pool).await?;

    // Process in batches of 1000 to avoid excessive memory usage
    let batch_size = 1000;
    let total_batches = (rows.len() + batch_size - 1) / batch_size;

    for batch_idx in 0..total_batches {
        let start_idx = batch_idx * batch_size;
        let end_idx = std::cmp::min(start_idx + batch_size, rows.len());
        let batch = &rows[start_idx..end_idx];

        let mut tx = pool.begin().await?;

        for row in batch {
            let id: i64 = row.get("id");
            let path: String = row.get("path");
            let filename: String = row.get("filename");

            // Generate trigrams for path and filename
            let path_trigrams = generate_trigrams(&path);
            let filename_trigrams = generate_trigrams(&filename);

            // Combine trigrams, remove duplicates
            let mut all_trigrams = path_trigrams;
            for trigram in filename_trigrams {
                if !all_trigrams.contains(&trigram) {
                    all_trigrams.push(trigram);
                }
            }

            // Insert trigrams
            for trigram in all_trigrams {
                sqlx::query(
                    r#"INSERT INTO path_trigrams (path_id, trigram) VALUES (?, ?)"#
                )
                .bind(id)
                .bind(&trigram)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
    }

    Ok(())
}

