mod api;
mod server_handler;

use std::collections::HashMap;

use actix_web::{
    self, App, HttpServer,
    web::{self},
};
use substrate_core::server::Server;
use tokio::sync::{RwLock, mpsc};

use crate::server_handler::ServerHandler;

pub type SharedServers = web::Data<RwLock<HashMap<String, Server>>>;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    #[cfg(feature = "logging")]
    tracing::info!("initialized tracing subscriber");
    tracing_subscriber::fmt::init();

    let (tx, rx) = mpsc::channel(100);

    let handler = ServerHandler {
        servers: HashMap::new(),
        rx,
    };

    tokio::spawn(handler.run());

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    /*.service(api::create_server::create_server)
                    .service(api::remove_server::remove_server)
                    .route("/ws", web::get().to(api::socket_control::socket_control)),*/
                    .route("/ws", web::get().to(api::socket_control::socket_control))
                    .service(api::create_server::create_server),
            )
            .app_data(web::Data::new(tx.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
