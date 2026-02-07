use crate::{api::control_msg::ControlMessage, indices::get_name_by_index, server::Servers};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{self, Payload},
};
use actix_ws::Message;
use futures_util::StreamExt;
use std::sync::Arc;

#[cfg(feature = "logging")]
use tracing::{error, info};

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
                                Ok(_) => {
                                    server.listen_to_server();
                                }
                                Err(e) => {
                                    #[cfg(feature = "logging")]
                                    error!("Failed to start server: {:?}", e);

                                    eprintln!("Failed to start server {}: {:?}", name, e);
                                }
                            }
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

                        ControlMessage::GetConsoleOutput { server_id } => {
                            // Access server and send console output
                        }

                        ControlMessage::Fail => {}
                    }
                }
            }
        }
    });

    Ok(res)
}
