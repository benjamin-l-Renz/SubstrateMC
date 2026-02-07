use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::RwLock,
};

use crate::errors::error::ApiError;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

#[derive(serde::Deserialize)]
pub struct ServerConfig {
    java_version: String,
}

pub struct Servers {
    pub servers: HashMap<u32, McServer>,
}

pub enum ServerState {
    Stopped,
    Running(tokio::process::Child),
}

impl PartialEq for ServerState {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (ServerState::Stopped, ServerState::Stopped))
    }
}

pub struct McServer {
    pub child: ServerState,
    pub history: Arc<RwLock<VecDeque<String>>>,
}

pub const MAX_LINES: usize = 500;

impl McServer {
    pub fn new() -> McServer {
        Self {
            child: ServerState::Stopped,
            history: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_LINES))),
        }
    }

    /// Starts the Minecraft server process.
    pub async fn start_server(&mut self, name: &str) -> Result<(), ApiError> {
        if self.child != ServerState::Stopped {
            println!("Cant start already running server");
            return Err(ApiError::InternalServerError);
        }

        let jar_dir = std::env::current_dir()?.join("servers").join(name);

        let config: ServerConfig =
            serde_json::from_str(&std::fs::read_to_string(jar_dir.join("config.json"))?)?;

        let child = tokio::process::Command::new(format!(
            "../../runtime/java-{}/bin/java",
            config.java_version
        ))
        .arg("-jar")
        .arg("server.jar")
        .arg("nogui")
        .current_dir(jar_dir)
        .spawn()?;

        self.child = ServerState::Running(child);

        Ok(())
    }

    /// Stops the Minecraft server process.
    pub async fn stop_server(&mut self) -> Result<(), ApiError> {
        if self.child == ServerState::Stopped {
            println!("Cant stop already stopped server");
            return Err(ApiError::InternalServerError);
        }

        if let ServerState::Running(child) = &mut self.child {
            child.kill().await?;
            child.wait().await?;
        }
        Ok(())
    }

    pub fn listen_to_server(&mut self) {
        if let ServerState::Running(child) = &mut self.child {
            if let Some(stdout) = child.stdout.take() {
                let history = self.history.clone();

                tokio::spawn(async move {
                    let mut reader = BufReader::new(stdout);
                    let mut line = String::with_capacity(512);

                    while reader.read_line(&mut line).await.unwrap() > 0 {
                        let line_to_store = std::mem::take(&mut line);
                        {
                            let mut hist = history.write().await;

                            if hist.len() > MAX_LINES {
                                hist.pop_back();
                            }
                            hist.push_front(line_to_store);

                            line.clear();
                        }
                    }
                });
            }
        }
    }

    pub async fn clear_history(&mut self) {
        self.history.write().await.clear();
    }
}
