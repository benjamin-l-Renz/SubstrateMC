#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
/// Represents control messages for managing Minecraft servers.
/// These messages are deserialized from JSON or MessagePack and used to control the server.
pub enum ControlMessage {
    StartServer {
        server_name: String,
    },
    StopServer {
        server_name: String,
    },
    GetConsoleOutput {
        server_name: String,
    },
    SendCommand {
        server_name: String,
        command: String,
    },
    Fail,
}

impl PartialEq for ControlMessage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ControlMessage::StartServer { server_name: name1 },
                ControlMessage::StartServer { server_name: name2 },
            ) => name1 == name2,
            (
                ControlMessage::StopServer { server_name: name1 },
                ControlMessage::StopServer { server_name: name2 },
            ) => name1 == name2,
            _ => false,
        }
    }
}
