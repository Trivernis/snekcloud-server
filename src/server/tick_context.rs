use crate::utils::result::SnekcloudError;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use vented::event::Event;
use vented::server::data::{Node, NodeData};
use vented::utils::sync::AsyncValue;

#[derive(Clone)]
pub struct TickContext {
    nodes: Arc<Mutex<HashMap<String, NodeData>>>,
    event_sender: Sender<EventInvocation>,
    node_id: String,
}

pub struct EventInvocation {
    pub result: AsyncValue<(), SnekcloudError>,
    pub event: Event,
    pub target_node: String,
}

impl TickContext {
    pub fn new(
        node_id: String,
        sender: Sender<EventInvocation>,
        nodes: Arc<Mutex<HashMap<String, NodeData>>>,
    ) -> Self {
        Self {
            nodes,
            node_id,
            event_sender: sender,
        }
    }

    pub fn emit<S: ToString>(
        &mut self,
        target_node: S,
        event: Event,
    ) -> AsyncValue<(), SnekcloudError> {
        let value = AsyncValue::new();
        self.event_sender
            .send(EventInvocation {
                event,
                target_node: target_node.to_string(),
                result: AsyncValue::clone(&value),
            })
            .unwrap();

        value
    }

    /// Returns a copy of the nodes of the server
    #[allow(dead_code)]
    pub fn nodes(&self) -> Vec<Node> {
        self.nodes
            .lock()
            .values()
            .cloned()
            .map(Node::from)
            .collect()
    }

    pub fn living_nodes(&self) -> Vec<Node> {
        self.nodes
            .lock()
            .values()
            .cloned()
            .filter(|node| !node.is_dead())
            .map(Node::from)
            .collect()
    }

    #[allow(dead_code)]
    pub fn check_alive(&self, node_id: &String) -> bool {
        if let Some(node) = self.nodes.lock().get(node_id) {
            !node.is_dead()
        } else {
            false
        }
    }

    /// Returns the node
    pub fn node_id(&self) -> &String {
        &self.node_id
    }
}
