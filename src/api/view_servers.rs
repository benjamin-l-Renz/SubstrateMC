use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;

use actix_web::{HttpResponse, get, web};

use crate::server_handler::HandlerCommand;

#[get("/view_servers")]
pub async fn view_servers(
    sender: web::Data<tokio::sync::mpsc::Sender<HandlerCommand>>,
) -> HttpResponse {
    let (tx, rx): (Sender<Vec<(String, bool)>>, Receiver<Vec<(String, bool)>>) =
        tokio::sync::oneshot::channel();
    let _ = sender.send(HandlerCommand::ViewServers { sender: tx });

    let servers = rx.await.unwrap();

    HttpResponse::Ok().json(servers)
}
