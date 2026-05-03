use std::{collections::VecDeque, path::Path, process::Stdio};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Child,
    sync::{broadcast, mpsc},
};

use crate::{
    console::{ConsoleActor, ConsoleHandler, ConsoleSubscription},
    errors::error::SubstrateError,
};

pub enum ServerStatus {
    Stopped,
    Running(Child),
}

pub struct Server {
    pub child: ServerStatus,
    pub handler: Option<ConsoleHandler>,
    // pub java_version: String,
}

impl Server {
    pub fn new(/*java_version: String*/) -> Self {
        Self {
            child: ServerStatus::Stopped,
            handler: None,
            // java_version,
        }
    }

    pub fn start_server(&mut self, name: &str, current_dir: &Path) -> Result<(), SubstrateError> {
        let server_dir = current_dir.join("servers").join(name);

        let process = tokio::process::Command::new("./run.sh")
            .current_dir(server_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.child = ServerStatus::Running(process);

        let (tx, rx) = mpsc::channel(32);
        let (broadcaster, _) = broadcast::channel(100);

        let actor = ConsoleActor {
            receiver: rx,
            history: VecDeque::new(),
            sender: broadcaster,
        };

        let handler = ConsoleHandler { sender: tx };

        self.handler = Some(handler);

        // Cloning only once (the tx and then implement internal tx logic no console handler)
        let handler = match self.handler.clone() {
            None => {
                return Err(SubstrateError::NotFound {
                    resource: "Could not clone handler".to_string(),
                });
            }

            Some(handler) => handler,
        };
        if let ServerStatus::Running(child) = &mut self.child
            && let Some(stdout) = child.stdout.take()
        {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout);
                let mut line = String::with_capacity(512);
                while reader.read_line(&mut line).await.unwrap() > 0 {
                    let line = std::mem::take(&mut line);
                    handler.send_line(line).await.unwrap();
                }
            });
        }

        tokio::spawn(actor.run());

        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<(), SubstrateError> {
        if let ServerStatus::Running(child) = &mut self.child {
            child.kill().await?;
            child.wait().await?;
            self.child = ServerStatus::Stopped;

            return Ok(());
        }

        Err(SubstrateError::NotFound {
            resource: "Server is not running".to_string(),
        })
    }

    pub async fn subscribe(&mut self) -> Result<ConsoleSubscription, SubstrateError> {
        if let Some(handler) = self.handler.as_mut() {
            match handler.subscribe().await {
                Ok(subscription) => {
                    return Ok(subscription);
                }

                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(SubstrateError::NotFound {
            resource: "Could not find handler".to_string(),
        })
    }

    pub async fn send_command(&mut self, command: String) -> Result<(), SubstrateError> {
        if let ServerStatus::Running(child) = &mut self.child {
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(command.as_bytes())
                .await?;
            return Ok(());
        }
        Err(SubstrateError::NotFound {
            resource: "Server is not running".to_string(),
        })
    }
}
