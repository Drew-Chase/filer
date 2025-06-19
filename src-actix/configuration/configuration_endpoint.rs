use crate::configuration::configuration_data::Configuration;
use crate::configuration::upnp;
use crate::helpers::http_error::Result;
use actix_web::{HttpResponse, get, post, web};
use actix_web::{Responder, delete};
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
    // Store the old configuration for comparison
    let old_config = Configuration::get().clone();

    // Save the new configuration
    body.0.save()?;

    // Handle UPnP port forwarding based on configuration changes
    upnp::handle_config_change(&old_config, &body.0);

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
