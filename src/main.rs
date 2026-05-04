mod api;
mod server_handler;

use std::collections::HashMap;

use actix_web::{self, App, HttpServer, web};
use tokio::sync::mpsc;

use crate::server_handler::ServerHandler;

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
                    .route("/ws", web::get().to(api::socket_control::socket_control))
                    .service(api::create_server::create_server)
                    .service(api::view_servers::view_servers)
                    .service(api::delete_server::delete_server),
            )
            .app_data(web::Data::new(tx.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
