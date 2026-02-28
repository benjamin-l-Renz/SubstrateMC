mod api;

use actix_web::{App, HttpServer, web};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[cfg(feature = "logging")]
use tracing::info;

use substrate_core::server::Server;

pub type SharedServers = web::Data<RwLock<HashMap<String, Server>>>;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    #[cfg(feature = "logging")]
    tracing::info!("initialized tracing subscriber");
    tracing_subscriber::fmt::init();

    let shared_servers: SharedServers = web::Data::new(RwLock::new(HashMap::new()));

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
