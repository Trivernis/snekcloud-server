/*
 * snekcloud node based network
 * Copyright (C) 2020 trivernis
 * See LICENSE for more information
 */

use crate::modules::heartbeat::payloads::HeartbeatPayload;
use crate::modules::heartbeat::settings::HeartbeatSettings;
use crate::modules::Module;
use crate::server::tick_context::RunContext;
use crate::utils::result::SnekcloudResult;
use crate::utils::settings::get_settings;
use crate::utils::write_json_pretty;
use async_std::task;
use async_trait::async_trait;
use chrono::Local;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vented::event::Event;
use vented::server::VentedServer;

mod payloads;
pub mod settings;
const HEARTBEAT_BEAT_EVENT: &str = "heartbeat:beat";

#[derive(Serialize, Deserialize, Clone, Debug)]
enum NodeState {
    Alive,
    Dead,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct NodeInfo {
    ping: Option<u64>,
    state: NodeState,
    timestamp: String,
}

impl NodeInfo {
    fn alive(ping: u64) -> Self {
        Self {
            ping: Some(ping),
            state: NodeState::Alive,
            timestamp: Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        }
    }
    fn dead() -> Self {
        Self {
            ping: None,
            state: NodeState::Dead,
            timestamp: Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        }
    }
}

pub struct HeartbeatModule {
    settings: HeartbeatSettings,
    node_states: Arc<Mutex<HashMap<String, Vec<NodeInfo>>>>,
}

impl HeartbeatModule {
    pub fn new() -> Self {
        Self {
            settings: get_settings().modules.heartbeat,
            node_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Module for HeartbeatModule {
    fn name(&self) -> String {
        "HeartbeatModule".to_string()
    }

    fn init(&mut self, server: &mut VentedServer) -> SnekcloudResult<()> {
        server.on(HEARTBEAT_BEAT_EVENT, {
            let node_states = Arc::clone(&self.node_states);

            move |event| {
                let node_states = Arc::clone(&node_states);
                Box::pin(async move {
                    let payload = event.get_payload::<HeartbeatPayload>().unwrap();
                    let latency = payload.get_beat_time().elapsed().ok()?.as_millis();
                    log::debug!("Latency to node {} is {} ms", payload.node_id, latency);

                    let mut states = node_states.lock();
                    Self::insert_state(
                        &mut states,
                        payload.node_id,
                        NodeInfo::alive(latency as u64),
                    );

                    None
                })
            }
        });

        Ok(())
    }

    fn boxed(self) -> Box<dyn Module + Send + Sync> {
        Box::new(self)
    }

    async fn run(&mut self, context: RunContext) -> SnekcloudResult<()> {
        for node in context.nodes() {
            let mut context = context.clone();
            let node_states = Arc::clone(&self.node_states);
            let interval = self.settings.interval();

            task::spawn(async move {
                loop {
                    Self::send_heartbeat(&mut context, &node.id, Arc::clone(&node_states)).await;

                    if !context.check_alive(&node.id) {
                        let start = Instant::now();
                        while !context.check_alive(&node.id) {
                            task::sleep(Duration::from_secs(10)).await;
                            if start.elapsed() > interval * 100 {
                                break;
                            }
                        }
                    } else {
                        task::sleep(interval).await
                    }
                }
            });
        }
        loop {
            if let Some(path) = &self.settings.output_file {
                let states = self.node_states.lock();
                if let Err(e) = write_json_pretty(path, &*states) {
                    log::error!("Failed to write output states to file: {}", e)
                }
            }
            task::sleep(self.settings.interval()).await
        }
    }
}

impl HeartbeatModule {
    fn insert_state(states: &mut HashMap<String, Vec<NodeInfo>>, id: String, state: NodeInfo) {
        lazy_static! {
            static ref MAX_RECORDS: usize = get_settings().modules.heartbeat.max_record_history;
        }
        if let Some(states) = states.get_mut(&id) {
            if states.len() > *MAX_RECORDS {
                states.remove(0);
            }
            states.push(state);
        } else {
            states.insert(id, vec![state]);
        }
    }

    async fn send_heartbeat(
        context: &mut RunContext,
        target: &String,
        states: Arc<Mutex<HashMap<String, Vec<NodeInfo>>>>,
    ) {
        log::trace!("Sending heartbeat to {}...", target);
        let mut value = context
            .emit(
                target.clone(),
                Event::with_payload(
                    HEARTBEAT_BEAT_EVENT,
                    &HeartbeatPayload::now(context.node_id().clone()),
                ),
            )
            .await;

        match value
            .get_value_with_timeout_async(Duration::from_secs(60))
            .await
        {
            Some(Err(e)) => {
                log::debug!("Node {} is not reachable: {}", target, e);
                Self::insert_state(&mut *states.lock(), target.clone(), NodeInfo::dead());
            }
            None => {
                log::debug!("Node {} is not reachable: Timeout", target);
                Self::insert_state(&mut *states.lock(), target.clone(), NodeInfo::dead());
            }
            _ => {}
        }
    }
}
