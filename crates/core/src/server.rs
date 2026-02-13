use std::{path::Path, process::Stdio};

use crate::errors::api_error::ApiError;
use tokio::process::Child;

#[derive(Debug)]
pub enum ServerStatus {
    Stopped,
    Running(Child),
}

impl PartialEq for ServerStatus {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ServerStatus::Stopped, ServerStatus::Stopped)
        )
    }
}

#[derive(Debug)]
pub struct Server<'a> {
    pub name: &'a str,
    pub java_version: &'a str,
    pub child: ServerStatus,
}

impl<'a> Server<'a> {
    pub fn new(name: &'a str, java_version: &'a str) -> Self {
        Self {
            name,
            child: ServerStatus::Stopped,
            java_version,
        }
    }

    pub fn start_server(&mut self, current_dir: &Path) -> Result<(), ApiError> {
        if self.child != ServerStatus::Stopped {
            return Err(ApiError::InternalServerError);
        }
        let server_dir = current_dir.join("servers").join(self.name);

        let process = tokio::process::Command::new(format!(
            "../../runtime/java-{}/bin/java",
            self.java_version
        ))
        .arg("-jar")
        .arg("server.jar")
        .arg("nogui")
        .current_dir(server_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

        self.child = ServerStatus::Running(process);

        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<(), ApiError> {
        if self.child == ServerStatus::Stopped {
            return Err(ApiError::InternalServerError);
        }

        if let ServerStatus::Running(child) = &mut self.child {
            child.kill().await?;
            child.wait().await?;

            self.child = ServerStatus::Stopped;
        }

        Ok(())
    }

    /*pub async fn listen_to_server() -> Result<(), ApiError> {
        let (tx, rx) = mpsc::channel(32);
        let (broadcaster, _) = broadcast::channel(100);

        let actor = ConsoleActor {
            history: VecDeque::new(),
            receiver: rx,
            sender: broadcaster,
        };

        tokio::spawn(actor.run());

        let handler = ConsoleHandler { sender: tx };

        let subscription = handler.subscribe().await?;

        Ok(())
    }*/
}
