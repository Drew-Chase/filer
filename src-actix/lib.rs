use crate::filesystem::filesystem_endpoint;
use crate::helpers::asset_endpoint::AssetsAppConfig;
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use anyhow::Result;
use log::*;
use serde_json::json;
use vite_actix::start_vite_server;
use crate::helpers::constants::{PORT, DEBUG};

mod filesystem;
mod helpers;


pub async fn run() -> Result<()> {
    pretty_env_logger::env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .format_timestamp(None)
        .init();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(|err, _req| {
                        let error = json!({ "error": format!("{}", err) });
                        actix_web::error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().json(error),
                        )
                        .into()
                    }),
            )
            .service(web::scope("api").configure(filesystem_endpoint::configure))
            .configure_frontend_routes()
    })
    .workers(4)
    .bind(format!("0.0.0.0:{port}", port = PORT))?
    .run();

    info!(
        "Starting {} server at http://127.0.0.1:{}...",
        if DEBUG { "development" } else { "production" },
        PORT
    );

    if DEBUG {
        start_vite_server().expect("Failed to start vite server");
    }

    let stop_result = server.await;
    debug!("Server stopped");

    Ok(stop_result?)
}
