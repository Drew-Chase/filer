use crate::auth::auth_data::User;
use crate::auth::permission_flags::PermissionFlags;
use crate::helpers::http_error::Result;
use actix_web::{delete, get, post, put, web, HttpResponse};
use anyhow::Error;
use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    password: String,
    permissions: Vec<String>,
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    password: Option<String>,
    permissions: Option<Vec<String>>,
}

#[derive(Serialize)]
struct UserResponse {
    id: u64,
    username: String,
    permissions: Vec<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        let permissions = user
            .permissions
            .iter()
            .map(|p| format!("{:?}", p))
            .collect();

        UserResponse {
            id: user.id,
            username: user.username,
            permissions,
        }
    }
}

fn parse_permissions(permissions: &[String]) -> Result<BitFlags<PermissionFlags>> {
    let mut flags = BitFlags::empty();

    for permission in permissions {
        match permission.as_str() {
            "Read" => flags |= PermissionFlags::Read,
            "Write" => flags |= PermissionFlags::Write,
            "Delete" => flags |= PermissionFlags::Delete,
            "Create" => flags |= PermissionFlags::Create,
            "Upload" => flags |= PermissionFlags::Upload,
            "Download" => flags |= PermissionFlags::Download,
            _ => Err(Error::msg("Invalid permission"))?,
        }
    }

    Ok(flags)
}

#[post("/users")]
async fn create_user(user_data: web::Json<CreateUserRequest>) -> Result<HttpResponse> {
    let permissions = parse_permissions(&user_data.permissions)?;

    if User::exists(&user_data.username).await? {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": format!("User {} already exists", user_data.username)
        })));
    }

    let user = User {
        id: 0,
        username: user_data.username.clone(),
        password: user_data.password.clone(),
        permissions,
    };

    user.create().await?;

    Ok(HttpResponse::Created().json(json!({
        "status": "created",
        "username": user_data.username
    })))
}

#[get("/users")]
async fn list_users() -> Result<HttpResponse> {
    let users = User::list().await?;
    let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(HttpResponse::Ok().json(user_responses))
}

#[get("/users/{username}")]
async fn get_user(path: web::Path<String>) -> Result<HttpResponse> {
    let username = path.into_inner();

    match User::get_by_username(&username).await? {
        Some(user) => Ok(HttpResponse::Ok().json(UserResponse::from(user))),
        None => Ok(HttpResponse::NotFound().json(json!({
            "error": format!("User {} not found", username)
        }))),
    }
}

#[put("/users/{username}")]
async fn update_user(
    path: web::Path<String>,
    user_data: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse> {
    let username = path.into_inner();

    let mut user = match User::get_by_username(&username).await? {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": format!("User {} not found", username)
            })));
        }
    };

    if let Some(password) = &user_data.password {
        user.password = password.clone();
    }

    if let Some(permissions) = &user_data.permissions {
        user.permissions = parse_permissions(permissions)?;
    }

    user.update().await?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "updated",
        "username": username
    })))
}

#[delete("/users/{username}")]
async fn delete_user(path: web::Path<String>) -> Result<HttpResponse> {
    let username = path.into_inner();
    let user = match User::get_by_username(&username).await? {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": format!("User {} not found", username)
            })));
        }
    };

    user.delete().await?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "deleted",
        "username": username
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(create_user)
            .service(list_users)
            .service(get_user)
            .service(update_user)
            .service(delete_user),
    );
}
