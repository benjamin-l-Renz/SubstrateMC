use actix_web::{HttpRequest, HttpResponse, web::Payload};
use actix_ws::Message;
use futures_util::StreamExt;
use substrate_core::errors::error::SubstrateError;

use crate::{
    SharedServers,
    api::control_msg::{ControlMessage, ServerEvent},
};

#[cfg(feature = "logging")]
use tracing::{error, info, warn};

pub async fn socket_control(
    req: HttpRequest,
    body: Payload,
    servers: SharedServers,
) -> Result<HttpResponse, SubstrateError> {
    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, body).unwrap();

    let current_dir = std::env::current_dir()?;

    // impl server struct handling and starting

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            let mut servers_map = servers.write().await;
            let resp = match &msg {
                Message::Text(t) => {
                    #[cfg(feature = "logging")]
                    info!("Received text message");

                    serde_json::from_str::<ControlMessage>(t).unwrap_or(ControlMessage::Fail)
                }

                Message::Binary(b) => {
                    #[cfg(feature = "logging")]
                    info!("Received binary message");

                    rmp_serde::from_slice::<ControlMessage>(b).unwrap_or(ControlMessage::Fail)
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

            if let Some(server) = servers_map.get_mut(name) {
                match &resp {
                    ControlMessage::StartServer { .. } => {
                        match server.start_server(name, &current_dir) {
                            Ok(_) => match server.listen_to_output().await {
                                Ok(_) => {}
                                Err(e) => {}
                            },
                            Err(e) => {}
                        }
                    }

                    ControlMessage::GetConsoleOutput { .. } => match server.subscribe().await {
                        Ok(mut subscription) => {
                            for line in subscription.history.iter() {
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

                            let mut session = session.clone();

                            tokio::spawn(async move {
                                loop {
                                    match subscription.tx.recv().await {
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
                        }
                        Err(e) => {}
                    },
                    _ => {}
                }
            }
        }
    });

    Ok(res)
}
