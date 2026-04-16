use crate::server_handler::{HandlerCommand, ServerCreateRequest};
use actix_web::{
    HttpResponse, post,
    web::{self},
};
use substrate_core::errors::error::SubstrateError;
use tokio::sync::mpsc::Sender;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateServerConfig {
    pub servers: Vec<McServerConfig>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct McServerConfig {
    pub name: String,
    pub java_version: String,
}

#[post("/create_server")]
pub async fn create_server(
    data: web::Json<ServerCreateRequest>,
    sender: web::Data<Sender<HandlerCommand>>,
) -> Result<impl actix_web::Responder, SubstrateError> {
    // TODO: return server name as bytes*/
    let data = data.into_inner();
    let name = data.name.clone();
    sender
        .send(HandlerCommand::CreateServer { request: data })
        .await
        .unwrap();

    let json = serde_json::to_string(&name).unwrap();

    Ok(HttpResponse::Ok().body(json))
}
