/*use crate::{
    api::open_socket::SharedServers, download_server::download_server, errors::error::ApiError,
};*/

use substrate_core::{
    download::download_server::{MinecraftConfig, download_server},
    errors::error::SubstrateError,
    server::Server,
};

use actix_web::{post, web};

#[cfg(feature = "logging")]
use tracing::error;

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
pub async fn create_server<'a>(
    data: web::Json<ServerCreateRequest>,
    servers: web::Data<SharedServers<'a>>,
) -> Result<impl actix_web::Responder, SubstrateError> {
    let data = data.into_inner();
    let servers = servers.into_inner();

    let current_dir = std::env::current_dir()?;

    let config_dir = current_dir.join("minecraft.json");

    if !tokio::fs::try_exists(&config_dir).await? {
        return Err(SubstrateError::NotFound {
            resource: "Could not find the minecraft.json".to_string(),
        });
    }

    let config_data = tokio::fs::File::open(config_dir).await?;

    let mc_config: MinecraftConfig = serde_json::from_reader(config_data.into_std().await)?;

    let (name, java_version) = download_server(
        &data.name,
        &data.loader,
        &data.minecraft_version,
        data.agree_eula,
        data.forced_java_version,
        current_dir,
        mc_config,
    )
    .await?;

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

    servers
        .write()
        .await
        .insert(name.clone(), Server::new(name, java_version));

    Ok(actix_web::HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(name_bytes))
}
