use crate::{
    api::{control_msg::ControlMessage, server_event::ServerEvent},
    indices::get_name_by_index,
    server::Servers,
};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{self, Payload},
};
use actix_ws::Message;
use futures_util::StreamExt;
use std::sync::Arc;

#[cfg(feature = "logging")]
use tracing::{error, info, warn};

use crate::errors::error::ApiError;

pub type SharedServers = Arc<tokio::sync::RwLock<Servers>>;

/// control the server via websocket
pub async fn ws_control(
    req: HttpRequest,
    body: Payload,
    servers: web::Data<SharedServers>,
) -> Result<HttpResponse, ApiError> {
    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let servers = servers.get_ref().clone();

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            // match message type send by client
            let resp = match msg {
                Message::Text(t) => {
                    #[cfg(feature = "logging")]
                    info!("Received text message");

                    serde_json::from_str::<ControlMessage>(&t).unwrap_or(ControlMessage::Fail)
                }

                Message::Binary(b) => {
                    #[cfg(feature = "logging")]
                    info!("Received binary message");

                    rmp_serde::from_slice::<ControlMessage>(&b).unwrap_or(ControlMessage::Fail)
                }

                Message::Close(_) => break,

                _ => continue,
            };

            // take server id and name without blocking through .await of a function drop the loack guard early so no deadlock occurs
            let val = {
                let guard = servers.read().await;

                let id = match resp {
                    ControlMessage::StartServer { server_id } => server_id,
                    ControlMessage::StopServer { server_id } => server_id,
                    ControlMessage::GetConsoleOutput { server_id } => server_id,
                    ControlMessage::SendCommand { server_id, .. } => server_id,
                    ControlMessage::Fail => continue,
                };

                guard
                    .servers
                    .get(&id)
                    .and_then(|_s| get_name_by_index(id))
                    .map(|name| (id, name))
            };

            // block here to ensure server operations are sequential using write lock
            if let Some((id, name)) = val {
                let mut servers_map = servers.write().await;

                if let Some(server) = servers_map.servers.get_mut(&id) {
                    match &resp {
                        ControlMessage::StartServer { .. } => {
                            match server.start_server(&name).await {
                                Ok(_) => {}
                                Err(e) => {
                                    #[cfg(feature = "logging")]
                                    error!("Failed to start server: {:?}", e);

                                    eprintln!("Failed to start server {}: {:?}", name, e);
                                }
                            }

                            server.listen_to_server();
                        }

                        ControlMessage::StopServer { .. } => match server.stop_server().await {
                            Ok(_) => {
                                server.clear_history().await;
                            }
                            Err(e) => {
                                #[cfg(feature = "logging")]
                                error!("Failed to stop server: {:?}", e);

                                eprintln!("Failed to stop server {}: {:?}", name, e);
                            }
                        },

                        ControlMessage::GetConsoleOutput { .. } => {
                            let history = {
                                let guard = server.history.read().await;
                                guard.iter().cloned().collect::<Vec<_>>()
                            };

                            for line in history.iter().rev() {
                                let event = ServerEvent::ConsoleLine { line };
                                if session
                                    .text(serde_json::to_string(&event).unwrap())
                                    .await
                                    .is_err()
                                {
                                    #[cfg(feature = "logging")]
                                    warn!("Client disconnected breaking loop");

                                    break;
                                }
                            }

                            let mut rx = server.tx.subscribe();

                            let mut session = session.clone();

                            tokio::spawn(async move {
                                loop {
                                    match rx.recv().await {
                                        Ok(line) => {
                                            let event = ServerEvent::ConsoleLine { line: &line };
                                            if session
                                                .text(serde_json::to_string(&event).unwrap())
                                                .await
                                                .is_err()
                                            {
                                                #[cfg(feature = "logging")]
                                                warn!("Client disconnected breaking loop");

                                                break;
                                            }
                                        }

                                        Err(tokio::sync::broadcast::error::RecvError::Lagged(
                                            _,
                                        )) => {
                                            // client failed to keep up with console output

                                            #[cfg(feature = "logging")]
                                            warn!("Client couldnt keep up with sender")
                                        }

                                        Err(e) => {
                                            #[cfg(feature = "logging")]
                                            error!("Failed to receive console output: {:?}", e);

                                            eprintln!("Failed to receive console output: {:?}", e);
                                        }
                                    }
                                }
                            });

                            // Access server and send console output
                        }

                        ControlMessage::Fail => {}

                        ControlMessage::SendCommand { command, .. } => {
                            match server.send_command(command).await {
                                Ok(_) => {
                                    #[cfg(feature = "logging")]
                                    info!("Command sent successfully");
                                }
                                Err(e) => {
                                    #[cfg(feature = "logging")]
                                    error!("Failed to send command: {:?}", e);

                                    eprintln!("Failed to send command: {:?}", e);
                                }
                            };
                        }
                    }
                }
            }
        }
    });

    Ok(res)
}

pub async fn socket_control(
    req: HttpRequest,
    body: Payload,
    servers: web::Data<SharedServers>,
) -> Result<HttpResponse, ApiError> {
    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            let resp = match msg {
                Message::Text(t) => {
                    #[cfg(feature = "logging")]
                    info!("Received text message");

                    serde_json::from_str::<ControlMessage>(&t).unwrap_or(ControlMessage::Fail)
                }

                Message::Binary(b) => {
                    #[cfg(feature = "logging")]
                    info!("Received binary message");

                    rmp_serde::from_slice::<ControlMessage>(&b).unwrap_or(ControlMessage::Fail)
                }

                Message::Close(_) => break,

                _ => continue,
            };

            let mut servers_map = servers.write().await;

            if let Some(server) = servers_map.servers.get_mut(&id) {
                match &resp {
                    _ => {}
                }
            }
        }
    });

    Ok(res)
}
