use std::sync::mpsc::Sender;
use vented::event::Event;
use std::sync::Arc;
use crate::utils::result::SnekcloudResult;
use vented::server::data::{AsyncValue, Node};
use parking_lot::Mutex;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TickContext {
    nodes: Arc<Mutex<HashMap<String, Node>>>,
    event_sender: Sender<EventInvocation>,
    node_id: String,
}

pub struct EventInvocation {
    pub result: AsyncValue<Arc<SnekcloudResult<()>>>,
    pub event: Event,
    pub target_node: String,
}

impl TickContext {
    pub fn new(node_id: String, sender: Sender<EventInvocation>, nodes: Arc<Mutex<HashMap<String, Node>>>) -> Self {
        Self {
            nodes,
            node_id,
            event_sender: sender,
        }
    }

    pub fn emit<S: ToString>(&mut self, target_node: S, event: Event) -> AsyncValue<Arc<SnekcloudResult<()>>> {
        let value = AsyncValue::new();
        self.event_sender.send(EventInvocation {
            event,
            target_node: target_node.to_string(),
            result: AsyncValue::clone(&value),
        }).unwrap();

        value
    }

    /// Returns a copy of the nodes of the server
    pub fn nodes(&self) -> Vec<Node> {
        self.nodes.lock().values().cloned().collect()
    }

    /// Returns the node
    pub fn node_id(&self) -> &String {
        &self.node_id
    }
}