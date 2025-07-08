use crate::arguments::FilerArguments;
use crate::auth::{auth_db, auth_endpoint};
use crate::configuration::configuration_data::Configuration;
use crate::configuration::configuration_endpoint;
use crate::configuration::upnp;
use crate::helpers::asset_endpoint::AssetsAppConfig;
use crate::helpers::constants::DEBUG;
use crate::internal_configuration::{ic_db, ic_endpoint};
use crate::io::fs::indexer::indexer_data::IndexerData;
use crate::io::fs::indexer::{indexer_data, indexer_db};
use crate::middleware::network::NetworkMiddleware;
use actix_web::{App, HttpResponse, HttpServer, middleware as actix_middleware, web};
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

pub mod arguments;
pub mod auth;
pub mod configuration;
pub mod helpers;
pub mod internal_configuration;
pub mod io;
pub mod middleware;

pub async fn run() -> Result<()> {
    pretty_env_logger::env_logger::builder().filter_level(LevelFilter::Debug).format_timestamp(None).init();

    let args = FilerArguments::parse();
    info!("Starting server...");
    if DEBUG {
        ProxyViteOptions::new().log_level(Info).build()?;
        std::thread::spawn(|| {
            let mut crashes: u8 = 0;
            loop {
                info!("Starting Vite server in development mode...");
                let status = start_vite_server().expect("Failed to start vite server").wait().expect("Vite server crashed!");
                if !status.success() {
                    crashes += 1;
                    error!("The vite server has crashed!");
                } else {
                    break;
                }
                if crashes > 5 {
                    error!("The vite server has crashed 5 times in a row. Vite will not be restarted.");
                    std::process::exit(1);
                }
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        });
        fs::create_dir_all("target/dev-env").await?;
        set_current_dir("target/dev-env")?;
    }

    Configuration::set_path("app-config.json")?;
    let config = Configuration::load()?;

    let port = args.port.unwrap_or(config.port);

    // Initialize UPnP functionality with the actual port being used
    if config.upnp_enabled {
        if let Err(upnp_error) = upnp::update_port_forwarding(port) {
            warn!("UPnP port forwarding failed: {}", upnp_error);
            // We continue with server startup even if UPnP fails
        }
    }

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
            .wrap(actix_middleware::Logger::default())
            .wrap(NetworkMiddleware) // Add our network middleware for CORS and IP filtering
            .app_data(web::JsonConfig::default().limit(4096).error_handler(|err, _req| {
                let error = json!({ "error": format!("{}", err) });
                actix_web::error::InternalError::from_response(err, HttpResponse::BadRequest().json(error)).into()
            }))
            .service(
                web::scope("/api")
                    .configure(auth_endpoint::configure)
                    .configure(filesystem_endpoint::configure)
                    .configure(configuration_endpoint::configure)
                    .configure(ic_endpoint::configure)
                    // Handle unmatched API endpoints
                    .default_service(web::to(|| async { HttpResponse::NotFound().json(json!({"error": "API endpoint not found"})) })),
            )
            .configure_frontend_routes()
    })
    .workers(4)
    .bind(format!("0.0.0.0:{}", port))?
    .run();

    info!("Starting {} server at http://127.0.0.1:{}...", if DEBUG { "development" } else { "production" }, port);

    let stop_result = server.await;
    debug!("Server stopped");

    // Clean up UPnP port forwarding
    upnp::cleanup();

    Ok(stop_result?)
}
