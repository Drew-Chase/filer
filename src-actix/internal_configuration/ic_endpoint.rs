use crate::helpers::http_error::Result;
use actix_web::{HttpResponse, Responder, get, post};
use serde_json::json;
use crate::internal_configuration::ic_data::InternalConfiguration;

#[get("/")]
pub async fn get_config() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(InternalConfiguration::get().await))
}

#[post("/complete-first-run-setup")]
pub async fn complete_first_run_setup() -> Result<impl Responder> {
	InternalConfiguration::default().set_has_done_first_run_setup(true).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/ic-config")
	        .service(get_config)
	        .service(complete_first_run_setup)
	        .default_service(actix_web::web::to(|| async {
            HttpResponse::NotFound().json(json!({
                "error": "API endpoint not found".to_string(),
            }))
        })),
    );
}
