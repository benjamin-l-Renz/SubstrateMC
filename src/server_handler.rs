use std::{collections::HashMap, fmt::Debug};

// TODO: Use Dashmap crate instaed of actor pattern so we can have quick lookups with minimal deadlocks in generall

use substrate_core::{
    download::server::{
        download_server::{ServerConfig, download_server},
        installer::{create_script::JavaFlags, loader::vanilla::VanillaInstaller},
    },
    server::{Server, ServerStatus},
};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::api::create_server::{CreateServerConfig, McServerConfig};

#[derive(serde::Deserialize, Debug)]
pub struct ServerCreateRequest {
    pub name: String,
    pub minecraft_version: String,
    pub loader: String,
    pub forced_java_version: Option<String>,
    pub agree_eula: bool,
    pub flags: JavaFlags,
}

#[derive(Debug)]
pub struct OutputSubscription {
    pub history: Vec<String>,
    pub rx: broadcast::Receiver<String>,
}

#[derive(Debug)]
pub enum HandlerCommand {
    StartServer {
        name: String,
    },
    StopServer {
        name: String,
    },
    GetOutput {
        name: String,
        sender: oneshot::Sender<OutputSubscription>,
    },
    SendCommand {
        name: String,
        command: String,
    },
    CreateServer {
        request: ServerCreateRequest,
    },

    ViewServers {
        sender: oneshot::Sender<Vec<(String, bool)>>,
    },

    DeleteServer {
        name: String,
    },
}

pub struct ServerHandler {
    pub servers: HashMap<String, Server>,
    pub rx: mpsc::Receiver<HandlerCommand>,
}

impl ServerHandler {
    pub async fn run(mut self) {
        let current_dir = std::env::current_dir().unwrap();

        let config_file = current_dir.join("servers.bin");

        if config_file.exists() {
            let bytes = tokio::fs::read(&config_file).await.unwrap();

            let config: CreateServerConfig = postcard::from_bytes(&bytes).unwrap();

            for server_config in config.servers {
                self.servers.insert(server_config.name, Server::new());
            }
        } else {
            let config = CreateServerConfig {
                servers: Vec::new(),
            };
            let bytes = postcard::to_allocvec(&config).unwrap();
            tokio::fs::write(&config_file, bytes).await.unwrap();
        }

        while let Some(msg) = self.rx.recv().await {
            match msg {
                HandlerCommand::StartServer { name } => {
                    let server = self.servers.get_mut(&name);

                    server.unwrap().start_server(&name, &current_dir).unwrap();
                }

                HandlerCommand::StopServer { name } => {
                    let server = self.servers.get_mut(&name);

                    server.unwrap().stop_server().await.unwrap();
                }

                HandlerCommand::GetOutput { name, sender } => {
                    let server = self.servers.get_mut(&name);

                    let subscription = server.unwrap().subscribe().await.unwrap();

                    let output = OutputSubscription {
                        history: subscription.history,
                        rx: subscription.tx,
                    };
                    sender.send(output).unwrap();
                }

                HandlerCommand::SendCommand { name, command } => {
                    let server = self.servers.get_mut(&name);
                    server.unwrap().send_command(command).await.unwrap();
                }

                HandlerCommand::CreateServer { request } => {
                    let installer = match request.loader.as_str() {
                        "vanilla" => VanillaInstaller {},
                        _ => {
                            panic!("...")
                        }
                    };

                    let (name, java_version) = download_server(
                        ServerConfig {
                            name: &request.name,
                            loader: &request.loader,
                            version: &request.minecraft_version,
                        },
                        request.agree_eula,
                        request.forced_java_version,
                        &current_dir,
                        &request.flags,
                        installer,
                    )
                    .await
                    .unwrap();

                    // java version is only needed for startup but we dont really need it after that only for the run file meaning we maybe could theoretically remove one clone when correctly structured
                    // Semms like java version isnt really needed after startup so i removed it we will see if it breaks anything

                    self.servers.insert(name.clone(), Server::new());

                    if !config_file.exists() {
                        let config = CreateServerConfig {
                            servers: vec![McServerConfig { name, java_version }],
                        };

                        let bytes = postcard::to_allocvec(&config).unwrap();

                        tokio::fs::write(&config_file, bytes).await.unwrap();
                    } else {
                        let bytes = tokio::fs::read(&config_file).await.unwrap();

                        let mut decoded: CreateServerConfig = postcard::from_bytes(&bytes).unwrap();

                        decoded.servers.push(McServerConfig { name, java_version });

                        let bytes = postcard::to_allocvec(&decoded).unwrap();

                        tokio::fs::write(&config_file, bytes).await.unwrap();
                    }
                }

                HandlerCommand::ViewServers { sender } => {
                    let result = self
                        .servers
                        .iter()
                        .map(|(name, server)| {
                            let running = matches!(server.child, ServerStatus::Running(_));
                            (name.clone(), running)
                        })
                        .collect::<Vec<(String, bool)>>();

                    let _ = sender.send(result);
                }

                HandlerCommand::DeleteServer { name } => {
                    if self.servers.remove(&name).is_some() {
                        let server_dir = current_dir.join("servers").join(&name);
                        if let Err(e) = tokio::fs::remove_dir_all(&server_dir).await {
                            eprintln!(
                                "Failed to delete server dir {}: {}",
                                server_dir.display(),
                                e
                            );
                        }
                    }
                }
            }
        }
    }
}
