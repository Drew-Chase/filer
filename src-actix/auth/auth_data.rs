use crate::auth::permission_flags::PermissionFlags;
use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};
use serde_hash::HashIds;
use sqlx::{FromRow, Row};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub permissions: Vec<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember: Option<bool>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub password: Option<String>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, HashIds)]
pub struct User {
    #[hash]
    pub id: u64,
    pub username: String,
    pub password: String,
    pub permissions: BitFlags<PermissionFlags>,
}

impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for User {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let id: i64 = row.try_get("id")?;
        let username: String = row.try_get("username")?;
        let password: String = row.try_get("password")?;
        let permissions_raw: i64 = row.try_get("permissions")?;
        
        let permissions = BitFlags::from_bits_truncate(permissions_raw as u8);
        
        Ok(User {
            id: id as u64,
            username,
            password,
            permissions,
        })
    }
}