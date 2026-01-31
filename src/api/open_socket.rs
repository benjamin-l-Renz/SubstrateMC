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
    let (res, _session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let servers = servers.get_ref().clone();

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            // match message type send by client
            let resp = match msg {
                Message::Text(t) => {
                    #[cfg(feature = "logging")]
                    info!("Received text message");

                    // TODO: handle error
                    serde_json::from_str::<ControlMessage>(&t).unwrap_or(ControlMessage::Fail)
                }

                Message::Binary(b) => {
                    #[cfg(feature = "logging")]
                    info!("Received binary message");

                    // TODO: handle error
                    rmp_serde::from_slice::<ControlMessage>(&b).unwrap_or(ControlMessage::Fail)
                }

                Message::Close(_) => break,

                _ => continue,
            };

            // take server id and name without blocking through .await of a function drop the loack guard early so no deadlock occurs
            let val = {
                let guard = servers.read().await;

                let id = match resp {
                    ControlMessage::StartServer { server_id } => server_id as i32,
                    ControlMessage::StopServer { server_id } => server_id as i32,
                    ControlMessage::Fail => continue,
                };

                guard
                    .servers
                    .get(&id)
                    .and_then(|_s| get_name_by_index(id as u32))
                    .map(|name| (id, name))
            };

            // block here to ensure server operations are sequential using write lock
            if let Some((id, name)) = val {
                let mut servers_map = servers.write().await;

                if let Some(server) = servers_map.servers.get_mut(&id) {
                    match &resp {
                        ControlMessage::StartServer { .. } => {
                            // If eula doesnt exist start the server two times

                            if let Err(e) = server.start_server(&name).await {
                                #[cfg(feature = "logging")]
                                error!("Failed to start server");
                                eprintln!("Failed to start server {}: {:?}", name, e);
                            }
                        }

                        ControlMessage::StopServer { .. } => {
                            if let Err(e) = server.stop_server().await {
                                #[cfg(feature = "logging")]
                                error!("Failed to stop server");

                                eprintln!("Failed to stop server {}: {:?}", name, e);
                            }
                        }

                        ControlMessage::Fail => {}
                    }
                }
            }
        }
    });

    Ok(res)
}
