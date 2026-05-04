use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;

use actix_web::{HttpResponse, get, web};

use crate::server_handler::HandlerCommand;

type OneShotSender = Sender<Vec<(String, bool)>>;
type OneShotReceiver = Receiver<Vec<(String, bool)>>;

#[get("/view_servers")]
pub async fn view_servers(
    sender: web::Data<tokio::sync::mpsc::Sender<HandlerCommand>>,
) -> HttpResponse {
    let (tx, rx): (OneShotSender, OneShotReceiver) = tokio::sync::oneshot::channel();
    sender
        .send(HandlerCommand::ViewServers { sender: tx })
        .await
        .unwrap();

    let servers = rx.await.unwrap();

    HttpResponse::Ok().json(servers)
}
