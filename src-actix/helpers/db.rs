use log::LevelFilter;
use sqlx::sqlite::SqliteSynchronous::Normal;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{ConnectOptions, SqlitePool};
use std::str::FromStr;

pub async fn create_pool() -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite:./app.db")?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .log_statements(LevelFilter::Trace)
        .synchronous(Normal);
    let pool = SqlitePoolOptions::new().max_connections(10).connect_with(options).await?;
    Ok(pool)
}
