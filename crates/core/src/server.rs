use std::{path::Path, process::Stdio};

use crate::{console::ConsoleHandler, errors::error::SubstrateError};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Child,
};

#[derive(Debug)]
/// State of a Minecraft server.
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
/// Server wrapper struct containing information about a Minecraft server.
///
/// # Fields
///
/// * `name` - The name of the server.
/// * `java_version` - The version of Java used by the server.
/// * `child` - The status of the server.
pub struct Server {
    pub name: String,
    pub java_version: String,
    pub child: ServerStatus,
}

impl Server {
    /// Creates a new instance of the Server struct.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the server.
    /// * `java_version` - The version of Java that will be used by the server.
    pub fn new(name: String, java_version: String) -> Self {
        Self {
            name,
            child: ServerStatus::Stopped,
            java_version,
        }
    }

    /// Starts the Minecraft server.
    ///
    /// Creates a new process to run the Minecraft server. and sets the status to Running with the process child.
    ///
    /// # Arguments
    ///
    /// * `current_dir` - The current project root directory.
    pub fn start_server(&mut self, current_dir: &Path) -> Result<(), SubstrateError> {
        if self.child != ServerStatus::Stopped {
            return Err(SubstrateError::McServerError {
                message: "Minecraft server already running".to_string(),
            });
        }
        let server_dir = current_dir.join("servers").join(&self.name);

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

    /// Stops the Minecraft server.
    ///
    /// Kills the server process. and sets the status to Stopped.
    pub async fn stop_server(&mut self) -> Result<(), SubstrateError> {
        if self.child == ServerStatus::Stopped {
            return Err(SubstrateError::McServerError {
                message: "Minecraft server already stopped".to_string(),
            });
        }

        if let ServerStatus::Running(child) = &mut self.child {
            child.kill().await?;
            child.wait().await?;

            self.child = ServerStatus::Stopped;
        }

        Ok(())
    }

    pub async fn listen_to_output(
        &mut self,
        handler: ConsoleHandler,
    ) -> Result<(), SubstrateError> {
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
