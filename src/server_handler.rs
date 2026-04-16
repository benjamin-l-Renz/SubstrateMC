use std::collections::HashMap;

use substrate_core::{
    download::server::{
        download_server::{ServerConfig, download_server},
        installer::{create_script::JavaFlags, loader::vanilla::VanillaInstaller},
    },
    server::Server,
};
use tokio::sync::{broadcast, mpsc};

use crate::api::create_server::{CreateServerConfig, McServerConfig};

#[derive(serde::Deserialize)]
pub struct ServerCreateRequest {
    pub name: String,
    pub minecraft_version: String,
    pub loader: String,
    pub forced_java_version: Option<String>,
    pub agree_eula: bool,
    pub flags: JavaFlags,
}

pub struct OutputSubscription {
    pub history: Vec<String>,
    pub rx: broadcast::Receiver<String>,
}

pub enum HandlerCommand {
    StartServer {
        name: String,
    },
    StopServer {
        name: String,
    },
    GetOutput {
        name: String,
        sender: mpsc::Sender<OutputSubscription>,
    },
    SendCommand {
        name: String,
        command: String,
    },
    CreateServer {
        request: ServerCreateRequest,
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
                self.servers
                    .insert(server_config.name, Server::new(server_config.java_version));
            }
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
                    sender.send(output).await.unwrap();
                }

                HandlerCommand::SendCommand { name, command } => {
                    let server = self.servers.get_mut(&name);
                    server
                        .unwrap()
                        .handler
                        .as_mut()
                        .unwrap()
                        .send_line(command)
                        .await
                        .unwrap();
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
                    //

                    self.servers
                        .insert(name.clone(), Server::new(java_version.clone()));

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
            }
        }
    }
}
