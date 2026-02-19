mod api;

use actix_web::{App, HttpServer, web};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[cfg(feature = "logging")]
use tracing::info;

use substrate_core::server::Server;

pub type SharedServers<'a> = RwLock<HashMap<String, Server>>;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    #[cfg(feature = "logging")]
    tracing_subscriber::fmt::init();

    #[cfg(feature = "logging")]
    tracing::info!("initialized tracing subscriber");
    // initialize_index().await.expect("Failed to init index.bin");

    /*let servers = SharedServers::new(RwLock::new(Servers {
        servers: HashMap::new(),
    }));*/

    let shared_servers: SharedServers = RwLock::new(HashMap::new());

    #[cfg(feature = "logging")]
    info!("Starting ElectronMC on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .service(api::create_server::create_server)
                    .service(api::remove_server::remove_server)
                    .route("/ws", web::get().to(api::open_socket::ws_control)),
            )
            .app_data(web::Data::new(shared_servers))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
