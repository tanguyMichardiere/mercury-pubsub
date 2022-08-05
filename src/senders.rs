use crate::models::channel::Channel;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Senders(HashMap<Uuid, broadcast::Sender<Value>>);

impl Senders {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&mut self, channel: &Channel) -> broadcast::Sender<Value> {
        match self.0.get(&channel.id) {
            Some(sender) => sender.clone(),
            None => {
                let (sender, _) = broadcast::channel(16);
                self.0.insert(channel.id, sender.clone());
                sender
            }
        }
    }

    pub fn get_receiver(&mut self, channel: &Channel) -> broadcast::Receiver<Value> {
        self.get(channel).subscribe()
    }
}

impl Default for Senders {
    fn default() -> Self {
        Self::new()
    }
}
