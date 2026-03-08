mod api;

use std::collections::HashMap;

use actix_web::{
    App, HttpServer,
    web::{self},
};

use substrate_core::server::Server;
use tokio::sync::RwLock;
#[cfg(feature = "logging")]
use tracing::info;

use crate::api::create_server::CreateServerConfig;

pub type SharedServers = web::Data<RwLock<HashMap<String, Server>>>;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    #[cfg(feature = "logging")]
    tracing::info!("initialized tracing subscriber");
    tracing_subscriber::fmt::init();

    // loading the servers into the list on startup or loading them in on starting or interacting

    let shared_servers: SharedServers = web::Data::new(RwLock::new(HashMap::new()));

    let current_dir = std::env::current_dir().unwrap();

    let config_file = current_dir.join("servers").join("servers.bin");

    if config_file.exists() {
        let bytes = tokio::fs::read(config_file).await.unwrap();

        let config: CreateServerConfig = rmp_serde::from_slice(&bytes).unwrap();

        for server_config in config.servers {
            shared_servers
                .write()
                .await
                .insert(server_config.name, Server::new(server_config.java_version));
        }
    }

    #[cfg(feature = "logging")]
    info!("Starting ElectronMC on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .service(api::create_server::create_server)
                    .service(api::remove_server::remove_server)
                    .route("/ws", web::get().to(api::socket_control::socket_control)),
            )
            .app_data(shared_servers.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
