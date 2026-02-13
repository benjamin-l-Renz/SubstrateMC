use std::collections::VecDeque;

use tokio::sync::{broadcast, mpsc, oneshot};

use crate::errors::api_error::ApiError;

const MAX_LINES: u16 = 500;

pub struct ConsoleSubscription {
    pub tx: broadcast::Receiver<String>,
    pub history: Vec<String>,
}

pub enum ConsoleMessage {
    SendLine(String),
    Subscribe(oneshot::Sender<ConsoleSubscription>),
}

pub struct ConsoleActor {
    pub receiver: mpsc::Receiver<ConsoleMessage>,
    pub history: VecDeque<String>,
    pub sender: broadcast::Sender<String>,
}

impl ConsoleActor {
    pub async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            match message {
                ConsoleMessage::SendLine(line) => {
                    if self.history.len() as u16 >= MAX_LINES {
                        self.history.pop_front();
                    }

                    // insert in history
                    self.history.push_back(line.clone());

                    // send line to the subscribed client
                    let _ = self.sender.send(line);
                }

                ConsoleMessage::Subscribe(response) => {
                    // get the history
                    let history: Vec<String> = self.history.iter().cloned().collect();

                    // create subscription element with the history and the receiver for the client
                    let subscription = ConsoleSubscription {
                        tx: self.sender.subscribe(),
                        history: history,
                    };

                    // send it to the client
                    let _ = response.send(subscription);
                }
            }
        }
    }
}

pub struct ConsoleHandler {
    pub sender: mpsc::Sender<ConsoleMessage>,
}

impl ConsoleHandler {
    pub async fn send_line(&self, line: String) -> Result<(), ApiError> {
        // Send an line as message
        if self
            .sender
            .send(ConsoleMessage::SendLine(line))
            .await
            .is_err()
        {
            return Err(ApiError::ChannelError);
        }

        Ok(())
    }

    pub async fn subscribe(&self) -> Result<ConsoleSubscription, ApiError> {
        // Create an channel
        let (tx, rx) = oneshot::channel();

        // send the sender to the actor
        if self
            .sender
            .send(ConsoleMessage::Subscribe(tx))
            .await
            .is_err()
        {
            return Err(ApiError::ChannelError);
        }

        // return the receiver
        Ok(rx.await.expect("Actor Crashed"))
    }
}
