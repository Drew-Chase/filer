use crate::io::fs::indexer::indexer_data::IndexerData;
use sqlx::sqlite::SqliteSynchronous::Normal;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Executor, Row, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;

pub async fn initialize() -> anyhow::Result<()> {
    let pool = create_pool().await?;
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS indexes
(
    path     TEXT UNIQUE PRIMARY KEY,
    filename TEXT    NOT NULL,
    mtime    INTEGER NOT NULL,
    ctime    INTEGER NOT NULL,
    size     INTEGER NOT NULL
);
		"#,
    )
    .await?;

    Ok(())
}
impl IndexerData {
    pub async fn insert_with_pool(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query(
            r#"insert into indexes (path, filename, mtime, ctime, size) values (?,?, ?, ?, ?)"#,
        )
        .bind(&self.path)
        .bind(&self.filename)
        .bind(self.mtime as i64)
        .bind(self.ctime as i64)
        .bind(self.size as i64)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn insert(&self) -> anyhow::Result<()> {
        let pool = create_pool().await?;
        self.insert_with_pool(&pool).await
    }

    pub async fn update_with_pool(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query(r#"update indexes set mtime = ?, ctime = ?, size = ? where path = ?"#)
            .bind(self.mtime as i64)
            .bind(self.ctime as i64)
            .bind(self.size as i64)
            .bind(&self.path)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(&self) -> anyhow::Result<()> {
        let pool = create_pool().await?;
        self.update_with_pool(&pool).await
    }

    pub async fn delete_with_pool(path: &str, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query(r#"delete from indexes where path = ?"#)
            .bind(path)
            .execute(pool)
            .await?;
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

    pub async fn search(query: impl AsRef<str>, filename_only: bool) -> anyhow::Result<Vec<Self>> {
        let pool = create_pool().await?;
        let result = sqlx::query_as::<_, IndexerData>(
            format!(
                r#"select * from indexes where {} like ?"#,
                if filename_only { "filename" } else { "path" }
            )
            .as_str(),
        )
        .bind(query.as_ref())
        .fetch_all(&pool)
        .await?;

        Ok(result)
    }
}

pub async fn create_pool() -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite:./app.db")?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(Normal);
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;
    Ok(pool)
}
