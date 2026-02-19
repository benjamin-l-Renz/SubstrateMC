use actix_web::post;
use actix_web::{HttpResponse, web};

//use crate::delete_server::delete_server;
//use crate::errors::error::ApiError;

use substrate_core::errors::error::SubstrateError;
use substrate_core::remove_server as delete_server;

#[derive(serde::Deserialize)]
struct RemoveForm {
    name: String,
}

#[post("/remove_server")]
pub async fn remove_server(
    data: web::Json<RemoveForm>,
) -> Result<impl actix_web::Responder, SubstrateError> {
    let data = data.into_inner();

    let project_dir = std::env::current_dir().unwrap();

    delete_server::delete_server(&data.name, &project_dir).await?;

    Ok(HttpResponse::Ok())
}
