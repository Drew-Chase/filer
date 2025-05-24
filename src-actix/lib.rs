use crate::auth::{auth_db, auth_endpoint};
use crate::helpers::asset_endpoint::AssetsAppConfig;
use crate::helpers::constants::{DEBUG, PORT};
use crate::io::fs::indexer::indexer_data::IndexerData;
use crate::io::fs::indexer::{indexer_data, indexer_db};
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use anyhow::Result;
use io::fs::filesystem_endpoint;
use log::Level::Info;
use log::*;
use serde_json::json;
use std::env::set_current_dir;
use vite_actix::proxy_vite_options::ProxyViteOptions;
use vite_actix::start_vite_server;

mod auth;
mod helpers;
pub mod io;

pub async fn run() -> Result<()> {
    pretty_env_logger::env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .format_timestamp(None)
        .init();

    info!("Starting server...");

    if DEBUG {
        ProxyViteOptions::new().log_level(Info).build()?;
        std::thread::spawn(|| {
            loop {
                info!("Starting Vite server in development mode...");
                let status = start_vite_server()
                    .expect("Failed to start vite server")
                    .wait()
                    .expect("Vite server crashed!");
                if !status.success() {
                    error!("The vite server has crashed!");
                } else {
                    break;
                }
            }
        });
        set_current_dir("target/wwwroot")?;
    }

    auth_db::initialize().await?;

    // Start file indexing and watcher in a separate task to avoid blocking server startup
    if !IndexerData::does_table_exist().await? {
        indexer_db::initialize().await?;
        tokio::spawn(async {
            if let Err(e) = indexer_data::index_all_files().await {
                error!("Error starting file indexer: {}", e);
            }
        });
    }

    if !DEBUG {
        tokio::spawn(async {
            // Start file watcher
            if let Err(e) = indexer_data::start_file_watcher().await {
                error!("Error starting file watcher: {}", e);
            }
        });
    }
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
                    .configure(filesystem_endpoint::configure)
                    // Handle unmatched API endpoints
                    .default_service(web::to(|| async {
                        HttpResponse::NotFound().json(json!({"error": "API endpoint not found"}))
                    })),
            )
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

    let stop_result = server.await;
    debug!("Server stopped");

    Ok(stop_result?)
}
