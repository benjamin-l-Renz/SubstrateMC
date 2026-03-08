use rmp_serde::{from_slice, to_vec};
use substrate_core::{
    download::download_server::{JavaFlags, download_server},
    errors::error::SubstrateError,
    server::Server,
};

use actix_web::{post, web};

#[cfg(feature = "logging")]
use tracing::{error, trace};

use crate::SharedServers;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateServerConfig {
    pub servers: Vec<McServerConfig>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct McServerConfig {
    pub name: String,
    pub java_version: String,
}

#[derive(serde::Deserialize)]
/// Represents the JSON payload for creating a new Minecraft server.
struct ServerCreateRequest {
    name: String,
    minecraft_version: String,
    loader: String,
    forced_java_version: Option<String>,
    agree_eula: bool,
    flags: JavaFlags,
}

/// API endpoint to create a new Minecraft server.
/// Expects a JSON payload with server details and returns the server index in MessagePack format.
#[post("/create_server")]
pub async fn create_server(
    data: web::Json<ServerCreateRequest>,
    servers: SharedServers,
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
        &current_dir,
        data.flags,
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

    {
        #[cfg(feature = "logging")]
        trace!("Creating write guard");
        let mut guard = servers.write().await;

        #[cfg(feature = "logging")]
        trace!("Insert server");
        guard.insert(name.clone(), Server::new(java_version.clone()));
    }

    let servers_file = &current_dir.join("servers").join("servers.bin");

    if !servers_file.exists() {
        let config = CreateServerConfig {
            servers: vec![McServerConfig { name, java_version }],
        };

        let bytes = to_vec(&config).unwrap();
        tokio::fs::write(servers_file, bytes).await?;
    } else {
        let bytes = tokio::fs::read(servers_file).await?;

        let mut decoded: CreateServerConfig = from_slice(&bytes).unwrap();

        decoded.servers.push(McServerConfig { name, java_version });

        let bytes = to_vec(&decoded).unwrap();

        tokio::fs::write(servers_file, bytes).await?;
    }

    Ok(actix_web::HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(name_bytes))
}
