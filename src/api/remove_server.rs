use actix_web::post;
use actix_web::{HttpResponse, web};

use crate::delete_server::delete_server;
use crate::errors::error::ApiError;

#[derive(serde::Deserialize)]
struct RemoveForm {
    name: String,
}

#[post("/remove_server")]
pub async fn remove_server(
    data: web::Json<RemoveForm>,
) -> Result<impl actix_web::Responder, ApiError> {
    let data = data.into_inner();

    delete_server(&data.name).await?;

    Ok(HttpResponse::Ok())
}
