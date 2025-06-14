use crate::configuration::configuration_data::Configuration;
use crate::helpers::http_error::Result;
use actix_web::{delete, Responder};
use actix_web::{HttpResponse, get, post, web};
use serde_json::json;
use std::collections::HashMap;

#[get("/")]
pub async fn get_config(query: web::Query<HashMap<String, String>>) -> Result<impl Responder> {
    let config = if query.get("reload").is_some() {
        &Configuration::load()?
    } else {
        Configuration::get()
    };
    Ok(HttpResponse::Ok().json(config))
}

#[post("/")]
pub async fn update_config(body: web::Json<Configuration>) -> Result<impl Responder> {
    body.0.save()?;
    Ok(HttpResponse::Ok().finish())
}

#[delete("/")]
pub async fn reset_config() -> Result<impl Responder> {
    Configuration::default().save()?;	
	Ok(HttpResponse::Ok().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/config")
            .service(get_config)
            .service(update_config)
            .service(reset_config)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
