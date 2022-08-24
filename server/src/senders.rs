use std::collections::HashMap;

use serde_json::Value;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::models::channel::Channel;

#[derive(Debug, Default, Clone)]
pub(crate) struct Senders(HashMap<Uuid, broadcast::Sender<Value>>);

impl Senders {
    pub(crate) fn get(&mut self, channel: &Channel) -> broadcast::Sender<Value> {
        match self.0.get(&channel.id) {
            Some(sender) => sender.clone(),
            None => {
                let (sender, _) = broadcast::channel(16);
                self.0.insert(channel.id, sender.clone());
                sender
            }
        }
    }

    pub(crate) fn get_receiver(&mut self, channel: &Channel) -> broadcast::Receiver<Value> {
        self.get(channel).subscribe()
    }
}
