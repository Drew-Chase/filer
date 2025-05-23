use crate::auth::auth_data::User;
use crate::auth::permission_flags::PermissionFlags;
use anyhow::Result;
use bcrypt::DEFAULT_COST;
use log::LevelFilter;
use serde_json::json;
use sqlx::sqlite::SqliteSynchronous::Normal;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{ConnectOptions, Error, Executor, SqlitePool};
use std::str::FromStr;

pub async fn initialize() -> Result<()> {
    let pool = create_pool().await?;
    pool.execute(
        r#"
CREATE TABLE IF NOT EXISTS users
(
    id          INTEGER PRIMARY KEY,
    username    TEXT    NOT NULL,
    password    TEXT    NOT NULL,
    permissions INTEGER NOT NULL
)
"#,
    )
    .await?;
    User::try_create_default_user(&pool).await?;
    pool.close().await;

    Ok(())
}

impl User {
    pub async fn try_create_default_user(pool: &SqlitePool) -> Result<()> {
        if !Self::exists_with_connection("filer", pool).await? {
            User {
                id: 0,
                username: "filer".to_string(),
                password: "filer".to_string(),
                permissions: PermissionFlags::Read
                    | PermissionFlags::Write
                    | PermissionFlags::Delete
                    | PermissionFlags::Create
                    | PermissionFlags::Upload
                    | PermissionFlags::Download,
            }
            .create_with_pool(pool)
            .await?;
        }

        Ok(())
    }
    pub async fn create(&self) -> Result<()> {
        let pool = create_pool().await?;
        self.create_with_pool(&pool).await
    }

    pub async fn create_with_pool(&self, pool: &SqlitePool) -> Result<()> {
        let permissions = self.permissions.bits_c() as i64;
        let password = bcrypt::hash(&self.password, DEFAULT_COST)?.to_string();
        sqlx::query("insert into users (username, password, permissions) values (?, ?, ?)")
            .bind(&self.username)
            .bind(&password)
            .bind(permissions)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn list() -> Result<Vec<Self>> {
        let pool = create_pool().await?;
        Self::list_with_pool(&pool).await
    }

    pub async fn list_with_pool(pool: &SqlitePool) -> Result<Vec<Self>> {
        let users = sqlx::query_as::<_, Self>("select * from users")
            .fetch_all(pool)
            .await?;
        Ok(users)
    }

    pub async fn delete(&self) -> Result<()> {
        let pool = create_pool().await?;
        self.delete_with_pool(&pool).await
    }

    pub async fn delete_with_pool(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query("delete from users where username = ?")
            .bind(&self.username)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(&self) -> Result<()> {
        let pool = create_pool().await?;
        self.update_with_pool(&pool).await
    }

    pub async fn update_with_pool(&self, pool: &SqlitePool) -> Result<()> {
        let permissions = self.permissions.bits_c() as i64;
        sqlx::query("update users set password = ?, permissions = ? where username = ?")
            .bind(&self.password)
            .bind(permissions)
            .bind(&self.username)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn reset_password(&self, new_password: impl AsRef<str>) -> Result<()> {
        let pool = create_pool().await?;
        self.reset_password_with_pool(new_password, &pool).await
    }

    pub async fn reset_password_with_pool(
        &self,
        new_password: impl AsRef<str>,
        pool: &SqlitePool,
    ) -> Result<()> {
        let password = bcrypt::hash(new_password.as_ref(), DEFAULT_COST)?.to_string();
        sqlx::query("update users set password = ? where username = ?")
            .bind(&password)
            .bind(&self.username)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_by_username(username: impl AsRef<str>) -> Result<Option<Self>> {
        let pool = create_pool().await?;
        Self::get_by_username_with_connection(username, &pool).await
    }

    pub async fn get_by_username_with_connection(
        username: impl AsRef<str>,
        pool: &SqlitePool,
    ) -> Result<Option<Self>> {
        let username = username.as_ref().to_string();
        match sqlx::query_as::<_, Self>("select * from users where username = ? limit 1")
            .bind(&username)
            .fetch_one(pool)
            .await
        {
            Ok(user) => Ok(Some(user)),
            Err(Error::RowNotFound) => Ok(None),
            Err(e) => Err(anyhow::Error::new(e)),
        }
    }

    pub async fn authenticate(
        username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Result<bool> {
        let pool = create_pool().await?;
        Self::authenticate_with_pool(username, password, &pool).await
    }

    pub async fn authenticate_with_pool(
        username: impl AsRef<str>,
        password: impl AsRef<str>,
        pool: &SqlitePool,
    ) -> Result<bool> {
        let username = username.as_ref().to_string();
        let password = password.as_ref().to_string();
        let user = Self::get_by_username_with_connection(username, pool).await?;
        if let Some(user) = user {
            let is_password_valid = bcrypt::verify(&password, &user.password)?;
            return Ok(is_password_valid);
        }
        Ok(true)
    }

    pub fn authenticate_with_session_token(
        &self,
        ip_address: impl AsRef<str>,
        host: impl AsRef<str>,
        session_token: impl AsRef<str>,
    ) -> Result<bool> {
        let session_token = session_token.as_ref().to_string();

        let json = json!({
            "id": self.id,
            "username": self.username,
            "password": self.password,
            "ip_address": ip_address.as_ref().to_string(),
            "host": host.as_ref().to_string(),
        })
        .to_string();
        let is_token_valid = bcrypt::verify(&json, &session_token)?;
        Ok(is_token_valid)
    }

    pub async fn exists_with_connection(
        username: impl AsRef<str>,
        pool: &SqlitePool,
    ) -> Result<bool> {
        let username = username.as_ref().to_string();
        Ok(
            !sqlx::query("select * from users where username = ? limit 1")
                .bind(&username)
                .fetch_all(pool)
                .await?
                .is_empty(),
        )
    }

    pub async fn exists(username: impl AsRef<str>) -> Result<bool> {
        let pool = create_pool().await?;
        Self::exists_with_connection(username, &pool).await
    }

    pub fn generate_session_token(
        &self,
        ip_address: impl AsRef<str>,
        host: impl AsRef<str>,
    ) -> Result<String> {
        let json = json!({
            "id": self.id,
            "username": self.username,
            "password": self.password,
            "ip_address": ip_address.as_ref().to_string(),
            "host": host.as_ref().to_string(),
        });
        let token = bcrypt::hash(json.to_string(), DEFAULT_COST)?;
        Ok(token)
    }
}

async fn create_pool() -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite:./app.db")?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .log_statements(LevelFilter::Trace)
        .synchronous(Normal);
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;
    Ok(pool)
}
