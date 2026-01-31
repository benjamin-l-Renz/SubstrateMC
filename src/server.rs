use crate::errors::error::ApiError;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
pub struct ServerConfig {
    java_version: String,
}

pub struct Servers {
    pub servers: HashMap<i32, McServer>,
}

pub enum ServerState {
    Stopped,
    Running(std::process::Child),
}

impl PartialEq for ServerState {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (ServerState::Stopped, ServerState::Stopped))
    }
}

pub struct McServer {
    pub child: ServerState,
}

impl McServer {
    pub fn new() -> McServer {
        Self {
            child: ServerState::Stopped,
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

        // TODO: load required java version from server config
        let child = std::process::Command::new(format!(
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
            child.kill()?;
            child.wait()?;
        }
        Ok(())
    }
}
