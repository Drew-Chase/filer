use crate::helpers::db::create_pool;
use sqlx::Executor;

pub async fn initialize() -> anyhow::Result<()> {
    let pool = create_pool().await?;
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS `internal_configuration`
(
    key   TEXT PRIMARY KEY UNIQUE,
    value TEXT DEFAULT NULL
)
		"#,
    )
    .await?;

    Ok(())
}

pub async fn get_all() -> anyhow::Result<std::collections::HashMap<String, String>> {
    let pool = create_pool().await?;
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM internal_configuration").fetch_all(&pool).await?;

    let mut map = std::collections::HashMap::new();
    for (key, value) in rows {
        map.insert(key, value);
    }

    Ok(map)
}

pub async fn get(key: &str) -> anyhow::Result<Option<String>> {
    let pool = create_pool().await?;
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM internal_configuration WHERE key = ?").bind(key).fetch_optional(&pool).await?;

    Ok(row.map(|(value,)| value))
}

pub async fn set(key: &str, value: &str) -> anyhow::Result<()> {
    let pool = create_pool().await?;
    sqlx::query("INSERT INTO internal_configuration (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
        .bind(key)
        .bind(value)
        .execute(&pool)
        .await?;

    Ok(())
}

pub async fn set_all(map: std::collections::HashMap<String, String>) -> anyhow::Result<()> {
    let pool = create_pool().await?;

    let mut items = vec![];
    for (key, value) in map {
        items.push(
            sqlx::query("INSERT INTO internal_configuration (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
                .bind(key.clone())
                .bind(value.clone())
                .execute(&pool),
        );
    }
    for item in items {
        item.await?;
    }

    Ok(())
}
