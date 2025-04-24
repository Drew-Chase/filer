use crate::auth::{auth_db, auth_endpoint};
use crate::filesystem::filesystem_endpoint;
use crate::helpers::asset_endpoint::AssetsAppConfig;
use crate::helpers::constants::{DEBUG, PORT};
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use anyhow::Result;
use log::Level::Info;
use log::*;
use serde_json::json;
use std::env::set_current_dir;
use vite_actix::proxy_vite_options::ProxyViteOptions;
use vite_actix::start_vite_server;

mod auth;
mod filesystem;
mod helpers;

pub async fn run() -> Result<()> {
    pretty_env_logger::env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .format_timestamp(None)
        .init();

    info!("Starting server...");

    if DEBUG {
        ProxyViteOptions::new().log_level(Info).build()?;
//        std::thread::spawn(|| {
//            loop {
//                info!("Starting Vite server in development mode...");
//                let status = start_vite_server()
//                    .expect("Failed to start vite server")
//                    .wait()
//                    .expect("Vite server crashed!");
//                if !status.success() {
//                    error!("The vite server has crashed!");
//                } else {
//                    break;
//                }
//            }
//        });
        set_current_dir("target/wwwroot")?;
    }

    auth_db::initialize().await?;
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
            .service(
                web::scope("/api")
                    .configure(auth_endpoint::configure)
                    .configure(filesystem_endpoint::configure),
            )
//            .configure_frontend_routes()
    })
    .workers(4)
    .bind(format!("0.0.0.0:{port}", port = PORT))?
    .run();

    info!(
        "Starting {} server at http://127.0.0.1:{}...",
        if DEBUG { "development" } else { "production" },
        PORT
    );

    let stop_result = server.await;
    debug!("Server stopped");

    Ok(stop_result?)
}
