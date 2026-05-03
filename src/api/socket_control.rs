use actix_web::{
    HttpRequest,
    web::{self, Payload},
};
use actix_ws::Message;
use futures_util::StreamExt;
use substrate_core::errors::error::SubstrateError;

use tokio::sync::{mpsc::Sender, oneshot};
#[cfg(feature = "logging")]
use tracing::info;

use crate::{
    api::control_message::{ControlMessage, ServerEvent},
    server_handler::HandlerCommand,
};

pub async fn socket_control(
    req: HttpRequest,
    body: Payload,
    /*servers: SharedServers,*/
    sender: web::Data<Sender<HandlerCommand>>,
) -> Result<actix_web::HttpResponse, SubstrateError> {
    let (response, session, mut msg_stream) = match actix_ws::handle(&req, body) {
        Ok(v) => v,
        Err(_) => {
            return Err(SubstrateError::SocketError {
                message: "failed to begin handling websocket traffic".to_string(),
            });
        }
    };

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            let resp = match &msg {
                Message::Text(t) => {
                    #[cfg(feature = "logging")]
                    info!("Received text message");

                    serde_json::from_str::<ControlMessage>(t).unwrap_or(ControlMessage::Fail)
                }

                Message::Close(_) => break,

                _ => continue,
            };

            let name: &str = {
                match &resp {
                    ControlMessage::StartServer { server_name } => server_name,
                    ControlMessage::StopServer { server_name } => server_name,
                    ControlMessage::GetConsoleOutput { server_name } => server_name,
                    ControlMessage::SendCommand { server_name, .. } => server_name,
                    ControlMessage::Fail => continue,
                }
            };

            /*let mut servers_map = servers.write().await;
            let server = servers_map.get_mut(name);*/

            match &resp {
                ControlMessage::StartServer { .. } => {
                    sender
                        .send(HandlerCommand::StartServer {
                            name: name.to_string(),
                        })
                        .await
                        .unwrap();
                }

                ControlMessage::StopServer { .. } => {
                    sender
                        .send(HandlerCommand::StopServer {
                            name: name.to_string(),
                        })
                        .await
                        .unwrap();
                }

                ControlMessage::GetConsoleOutput { .. } => {
                    let (tx, rx) = oneshot::channel();
                    sender
                        .send(HandlerCommand::GetOutput {
                            name: name.to_string(),
                            sender: tx,
                        })
                        .await
                        .unwrap();

                    let mut output_subscription = rx.await.unwrap();

                    let mut session = session.clone();

                    for line in output_subscription.history.iter() {
                        let event = ServerEvent::ConsoleLine { line };

                        if session
                            .text(serde_json::to_string(&event).unwrap())
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }

                    tokio::spawn(async move {
                        loop {
                            match output_subscription.rx.recv().await {
                                Ok(line) => {
                                    let event = ServerEvent::ConsoleLine { line: &line };

                                    if session
                                        .text(serde_json::to_string(&event).unwrap())
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }

                                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                    // client failed to keep up with console output
                                }
                                Err(e) => {
                                    eprintln!("Failed to receive console output: {:?}", e);
                                    break;
                                }
                            }
                        }
                    });
                }

                ControlMessage::SendCommand { command, .. } => {
                    sender
                        .send(HandlerCommand::SendCommand {
                            name: name.to_string(),
                            command: command.to_string(),
                        })
                        .await
                        .unwrap();
                }

                _ => {}
            }
        }
    });

    Ok(response)
}
