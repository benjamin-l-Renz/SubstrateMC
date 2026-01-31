use crate::{
    api::open_socket::SharedServers, download_server::download_server, errors::error::ApiError,
};
use actix_web::{post, web};

#[cfg(feature = "logging")]
use tracing::error;

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
) -> Result<impl actix_web::Responder, ApiError> {
    let data = data.into_inner();

    let index = download_server(
        &data.name,
        &data.minecraft_version,
        &data.loader,
        data.forced_java_version.as_deref(),
        servers,
        data.agree_eula,
    )
    .await?;

    let index_bytes = match rmp_serde::to_vec(&index) {
        Ok(bytes) => bytes,
        Err(err) => {
            #[cfg(feature = "logging")]
            error!("Failed to serialize server index: {}", err);
            return Err(ApiError::InternalServerError);
        }
    };

    Ok(actix_web::HttpResponse::Ok()
        .content_type("application/msgpack")
        .body(index_bytes))
}
