mod api;
mod check_java;
mod download;
mod download_java;
mod download_server;
mod errors;
mod indices;
mod server;
mod unpack;

use actix_web::{App, HttpServer, web};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::info;

use indices::initialize_index;

use crate::{api::open_socket::SharedServers, server::Servers};

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    #[cfg(feature = "logging")]
    tracing_subscriber::fmt::init();

    #[cfg(feature = "logging")]
    tracing::info!("initialized tracing subscriber");
    initialize_index().await.expect("Failed to init index.bin");

    let servers = SharedServers::new(RwLock::new(Servers {
        servers: HashMap::new(),
    }));

    #[cfg(feature = "logging")]
    info!("Starting ElectronMC on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .service(api::create_server::create_server)
                    .route("/ws", web::get().to(api::open_socket::ws_control)),
            )
            .app_data(web::Data::new(servers.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
