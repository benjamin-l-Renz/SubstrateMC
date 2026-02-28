use substrate_core::{
    download::download_server::download_server, errors::error::SubstrateError, server::Server,
};

use actix_web::{post, web};

#[cfg(feature = "logging")]
use tracing::{error, trace};

use crate::SharedServers;

#[derive(serde::Deserialize)]
/// Represents the JSON payload for creating a new Minecraft server.
struct ServerCreateRequest {
    name: String,
    minecraft_version: String,
    loader: String,
    forced_java_version: Option<String>,
    agree_eula: bool,
}

/// API endpoint to create a new Minecraft server.
/// Expects a JSON payload with server details and returns the server index in MessagePack format.
#[post("/create_server")]
pub async fn create_server(
    data: web::Json<ServerCreateRequest>,
    servers: web::Data<SharedServers>,
) -> Result<impl actix_web::Responder, SubstrateError> {
    #[cfg(feature = "logging")]
    trace!("Moving data into inner");
    let data = data.into_inner();

    #[cfg(feature = "logging")]
    trace!("Get current dir");
    let current_dir = std::env::current_dir()?;

    #[cfg(feature = "logging")]
    trace!("Downloading server...");
    let (name, java_version) = download_server(
        &data.name,
        &data.loader,
        &data.minecraft_version,
        data.agree_eula,
        data.forced_java_version,
        current_dir,
    )
    .await?;

    #[cfg(feature = "logging")]
    trace!("Move name to bytes");
    let name_bytes = match rmp_serde::to_vec(&name) {
        Ok(bytes) => bytes,
        Err(err) => {
            #[cfg(feature = "logging")]
            error!("Failed to serialize server index: {}", err);
            return Err(SubstrateError::ConversionError {
                details: "Failed to convert to message pack".to_string(),
            });
        }
    };

    #[cfg(feature = "logging")]
    trace!("Creating write guard");
    let mut guard = servers.write().await;

    #[cfg(feature = "logging")]
    trace!("Insert server");
    guard.insert(name, Server::new(java_version));

    #[cfg(feature = "logging")]
    trace!("Dropping write guard");
    drop(guard);

    Ok(actix_web::HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(name_bytes))
}
