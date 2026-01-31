#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
/// Represents control messages for managing Minecraft servers.
/// These messages are deserialized from JSON or MessagePack and used to control the server.
pub enum ControlMessage {
    StartServer { server_id: u32 },
    StopServer { server_id: u32 },
    Fail,
}

impl PartialEq for ControlMessage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ControlMessage::StartServer { server_id: id1 },
                ControlMessage::StartServer { server_id: id2 },
            ) => id1 == id2,
            (
                ControlMessage::StopServer { server_id: id1 },
                ControlMessage::StopServer { server_id: id2 },
            ) => id1 == id2,
            _ => false,
        }
    }
}
