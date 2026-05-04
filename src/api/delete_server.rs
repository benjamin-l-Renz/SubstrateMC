use std::sync::mpsc::Sender;

use actix_web::{HttpResponse, post, web};

use crate::server_handler::HandlerCommand;

#[post("/delete_server")]
pub async fn delete_server(
    name: web::Json<String>,
    sender: web::Data<Sender<HandlerCommand>>,
) -> HttpResponse {
    let name = name.into_inner();
    sender.send(HandlerCommand::DeleteServer { name }).unwrap();
    HttpResponse::Ok().body("Server deleted")
}
