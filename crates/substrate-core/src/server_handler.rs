use std::collections::HashMap;

use crate::server::Server;

pub struct ServerHandler {
    pub servers: HashMap<String, Server>,
}

impl ServerHandler {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }
}
