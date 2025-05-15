use std::cell::Ref;
use std::ops::Deref;
use crate::auth::auth_data::User;
use crate::auth::permission_flags::PermissionFlags;
use crate::helpers::http_error::Result;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web::dev::ConnectionInfo;
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
struct LoginRequest {
    username: String,
    password: String,
    remember: Option<bool>,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    username: String,
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

#[post("/login")]
async fn login(req: HttpRequest, login_data: web::Json<LoginRequest>) -> Result<HttpResponse> {
    let username = &login_data.username;
    let password = &login_data.password;
    let remember = login_data.remember.unwrap_or(false);
    
    let is_authenticated = User::authenticate(username, password).await?;
    
    if !is_authenticated {
        return Ok(HttpResponse::Unauthorized().json(json!({
            "error": "Invalid username or password"
        })));
    }
    
    let user = User::get_by_username(username).await?;
    if let Some(user) = user {
        let connection_info = req.connection_info();
        let ip = connection_info.peer_addr().unwrap_or("unknown");
        let host = connection_info.host().to_string();
        
        let token = user.generate_session_token(ip, host)?;
        
        // Prepare response
        let mut response = HttpResponse::Ok();
        
        // Set cookie expiration based on a remember flag
        if remember {
            // 30 days for "remember me"
            response.cookie(
                actix_web::cookie::Cookie::build("token", token.clone())
                    .path("/")
                    .max_age(actix_web::cookie::time::Duration::days(365))
                    .http_only(true)
                    .secure(true) // Only send over HTTPS
                    .same_site(actix_web::cookie::SameSite::Strict)
                    .finish()
            );
        } else {
            // Session cookie (expires when the browser closes)
            response.cookie(
                actix_web::cookie::Cookie::build("token", token.clone())
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .same_site(actix_web::cookie::SameSite::Strict)
                    .finish()
            );
        }

        Ok(response.json(LoginResponse {
            token,
            username: user.username,
        }))
    } else {
        Ok(HttpResponse::Unauthorized().json(json!({
            "error": "User not found"
        })))
    }
}

#[get("/validate-token")]
async fn validate_token(req: HttpRequest) -> Result<HttpResponse> {
    // Try to get the token from cookies
    if let Some(token_cookie) = req.cookie("token") {
        let token = token_cookie.value().to_string();
        let ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
        let host = req.connection_info().host().to_string();
        
        // Loop through all users to find one that validates with this token
        // This is not the most efficient approach but works for demonstration
        let users = User::list().await?;
        for user in users {
            if user.authenticate_with_session_token(&ip, &host, &token)? {
                return Ok(HttpResponse::Ok().json(json!({
                    "username": user.username,
                    "valid": true
                })));
            }
        }
    }
    
    // If we get here, either no token was found or it was invalid
    Ok(HttpResponse::Unauthorized().json(json!({
        "valid": false,
        "error": "Invalid or expired token"
    })))
}

#[post("/logout")]
async fn logout() -> HttpResponse {
    let mut response = HttpResponse::Ok();
    
    // Remove the token cookie by setting an expired cookie
    response.cookie(
        actix_web::cookie::Cookie::build("token", "")
            .path("/")
            .max_age(actix_web::cookie::time::Duration::seconds(-1))
            .http_only(true)
            .finish()
    );
    
    response.json(json!({ "status": "logged_out" }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(create_user)
            .service(list_users)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
            .service(login)
            .service(validate_token)
            .service(logout),
    );
}
