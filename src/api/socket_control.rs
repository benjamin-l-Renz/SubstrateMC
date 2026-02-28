use actix_web::{HttpRequest, HttpResponse, web, web::Payload};
use actix_ws::Message;
use futures_util::StreamExt;
use substrate_core::{console::ConsoleHandler, errors::error::SubstrateError};
use tokio::sync::{broadcast, mpsc};

use crate::{SharedServers, api::control_msg::ControlMessage};

#[cfg(feature = "logging")]
use tracing::{error, info};

pub async fn socket_control(
    req: HttpRequest,
    body: Payload,
    servers: web::Data<SharedServers>,
) -> Result<HttpResponse, SubstrateError> {
    let (res, _, mut msg_stream) = actix_ws::handle(&req, body).unwrap();

    let servers = servers.clone();

    let current_dir = std::env::current_dir()?;

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

            let name = {
                match &resp {
                    ControlMessage::StartServer { server_name } => server_name,
                    ControlMessage::StopServer { server_name } => server_name,
                    ControlMessage::GetConsoleOutput { server_name } => server_name,
                    ControlMessage::SendCommand { server_name, .. } => server_name,
                    ControlMessage::Fail => continue,
                }
            };

            let mut servers_map = servers.write().await;

            if let Some(server) = servers_map.get_mut(name) {
                match &resp {
                    // TODO: initialize history here
                    ControlMessage::StartServer { .. } => {
                        match server.start_server(&current_dir, name).await {
                            Ok(_) => {
                                server.listen_to_output().await.unwrap();
                            }
                            Err(e) => {
                                #[cfg(feature = "logging")]
                                error!("Failed to start server: {:?}", e);

                                eprintln!("Failed to start server {}: {:?}", name, e);
                            }
                        }
                    }

                    ControlMessage::StopServer { .. } => {
                        // TODO: Clean history
                        match server.stop_server().await {
                            Ok(_) => {}
                            Err(e) => {
                                #[cfg(feature = "logging")]
                                error!("Failed to stop server: {:?}", e);

                                eprintln!("Failed to stop server {}: {:?}", name, e);
                            }
                        }
                    }

                    ControlMessage::GetConsoleOutput { .. } => {}

                    _ => continue,
                }
            }
        }
    });

    Ok(res)
}
