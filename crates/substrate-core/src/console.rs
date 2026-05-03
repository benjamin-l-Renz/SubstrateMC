use std::collections::VecDeque;

use tokio::sync::{broadcast, mpsc, oneshot};

use crate::errors::error::SubstrateError;

const MAX_LINES: usize = 500;

/// Subscription payload sent to the client
///
/// # Fields
/// * `tx` - The receiver for the client.
/// * `history` - The history of the console.
pub struct ConsoleSubscription {
    pub tx: broadcast::Receiver<String>,
    pub history: Vec<String>,
}

/// Message sent to the console actor
///
/// # Variants
/// * `SendLine(String)` - Send a line to the console.
/// * `Subscribe(oneshot::Sender<ConsoleSubscription>)` - Subscribe to the console.
pub enum ConsoleMessage {
    SendLine(String),
    Subscribe(oneshot::Sender<ConsoleSubscription>),
}

/// Console actor which handles console messages and subscriptions.
pub struct ConsoleActor {
    pub receiver: mpsc::Receiver<ConsoleMessage>,
    pub history: VecDeque<String>,
    pub sender: broadcast::Sender<String>,
}

impl ConsoleActor {
    /// Run the console actor.
    pub async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            match message {
                // TODO: Send line should not be used for sending messages to server
                // it should be used for sending the stdout output
                ConsoleMessage::SendLine(line) => {
                    if self.history.len() >= MAX_LINES {
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
                        history,
                    };

                    // send it to the client
                    let _ = response.send(subscription);
                }
            }
        }
    }
}

/// Handler and helper for sending messages to the console actor.
#[derive(Clone)]
pub struct ConsoleHandler {
    pub sender: mpsc::Sender<ConsoleMessage>,
}

impl ConsoleHandler {
    /// Sends a line to the console actor.
    /// This should be called from the thread listening to console output.
    pub async fn send_line(&self, line: String) -> Result<(), SubstrateError> {
        // Send an line as message
        if self
            .sender
            .send(ConsoleMessage::SendLine(line))
            .await
            .is_err()
        {
            return Err(SubstrateError::ChannelSend);
        }

        Ok(())
    }

    /// Subscribes to the console actor and returns a subscription with the console history and an live receiver.
    pub async fn subscribe(&self) -> Result<ConsoleSubscription, SubstrateError> {
        // Create an channel
        let (tx, rx) = oneshot::channel();

        // send the sender to the actor
        if self
            .sender
            .send(ConsoleMessage::Subscribe(tx))
            .await
            .is_err()
        {
            return Err(SubstrateError::ChannelSend);
        }

        // return the receiver
        Ok(rx.await.expect("Actor Crashed"))
    }
}
