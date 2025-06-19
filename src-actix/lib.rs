use crate::arguments::FilerArguments;
use crate::auth::{auth_db, auth_endpoint};
use crate::configuration::configuration_data::Configuration;
use crate::helpers::asset_endpoint::AssetsAppConfig;
use crate::helpers::constants::DEBUG;
use crate::io::fs::indexer::indexer_data::IndexerData;
use crate::io::fs::indexer::{indexer_data, indexer_db};
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use anyhow::Result;
use clap::Parser;
use io::fs::filesystem_endpoint;
use log::Level::Info;
use log::*;
use serde_json::json;
use std::env::set_current_dir;
use tokio::fs;
use vite_actix::proxy_vite_options::ProxyViteOptions;
use vite_actix::start_vite_server;
use crate::configuration::configuration_endpoint;
use crate::internal_configuration::{ic_db, ic_endpoint};

mod arguments;
mod auth;
mod configuration;
pub(crate) mod helpers;
pub mod io;
mod internal_configuration;

pub async fn run() -> Result<()> {
    pretty_env_logger::env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .format_timestamp(None)
        .init();

    let args = FilerArguments::parse();

    info!("Starting server...");
    if DEBUG {
        ProxyViteOptions::new().log_level(Info).build()?;
        std::thread::spawn(|| {
            let mut crashes: u8 = 0;
            loop {
                info!("Starting Vite server in development mode...");
                let status = start_vite_server()
                    .expect("Failed to start vite server")
                    .wait()
                    .expect("Vite server crashed!");
                if !status.success() {
                    crashes += 1;
                    error!("The vite server has crashed!");
                } else {
                    break;
                }
                if crashes > 5 {
                    error!(
                        "The vite server has crashed 5 times in a row. Vite will not be restarted."
                    );
                }
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        });
        fs::create_dir_all("target/dev-env").await?;
        set_current_dir("target/dev-env")?;
    }

    Configuration::set_path("app-config.json")?;
    Configuration::load()?;
    let config = Configuration::get();

    let port = args.port.unwrap_or(config.port);

    auth_db::initialize().await?;
    ic_db::initialize().await?;

    // Start file indexing and watcher in a separate task to avoid blocking server startup
    if !args.disable_indexing && config.indexing_enabled {
        if !IndexerData::does_table_exist().await? {
            indexer_db::initialize().await?;
            tokio::spawn(async {
                if let Err(e) = indexer_data::index_all_files().await {
                    error!("Error starting file indexer: {}", e);
                }
            });
        }
        if !args.disable_filewatchers && config.file_watcher_enabled {
            tokio::spawn(async {
                // Start file watcher
                if let Err(e) = indexer_data::start_file_watcher().await {
                    error!("Error starting file watcher: {}", e);
                }
            });
        }
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
                    .configure(configuration_endpoint::configure)
                    .configure(ic_endpoint::configure)
                    // Handle unmatched API endpoints
                    .default_service(web::to(|| async {
                        HttpResponse::NotFound().json(json!({"error": "API endpoint not found"}))
                    })),
            )
            .configure_frontend_routes()
    })
    .workers(4)
    .bind(format!("0.0.0.0:{}", port))?
    .run();

    info!(
        "Starting {} server at http://127.0.0.1:{}...",
        if DEBUG { "development" } else { "production" },
        port
    );

    let stop_result = server.await;
    debug!("Server stopped");

    Ok(stop_result?)
}
