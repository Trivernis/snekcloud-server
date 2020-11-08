use crate::modules::Module;
use vented::server::VentedServer;
use crate::utils::result::SnekcloudResult;
use scheduled_thread_pool::ScheduledThreadPool;
use vented::result::{VentedResult};
use crate::modules::heartbeat::payloads::HeartbeatPayload;
use std::time::{Instant};
use vented::event::Event;
use crate::utils::settings::get_settings;
use crate::modules::heartbeat::settings::HeartbeatSettings;
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::utils::{write_json_pretty};

pub mod settings;
mod payloads;
const HEARTBEAT_BEAT_EVENT: &str = "heartbeat:beat";

#[derive(Serialize, Deserialize, Clone, Debug)]
enum NodeState {
    Alive,
    Dead
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct NodeInfo {
    ping: Option<u64>,
    state: NodeState,
}

impl NodeInfo {
    fn alive(ping: u64) -> Self {
        Self {
            ping: Some(ping),
            state: NodeState::Alive,
        }
    }
    fn dead() -> Self {
        Self {
            ping: None,
            state: NodeState::Dead,
        }
    }
}

pub struct HeartbeatModule {
    last_tick: Instant,
    settings: HeartbeatSettings,
    node_states: Arc<Mutex<HashMap<String, Vec<NodeInfo>>>>,
}

impl HeartbeatModule {
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            settings: get_settings().modules.heartbeat,
            node_states: Arc::new(Mutex::new(HashMap::new()))
        }
    }
}

impl Module for HeartbeatModule {
    fn name(&self) -> String {
        "HeartbeatModule".to_string()
    }

    fn init(&mut self, server: &mut VentedServer, pool: &mut ScheduledThreadPool) -> SnekcloudResult<()> {
        server.on(HEARTBEAT_BEAT_EVENT, {
            let node_states = Arc::clone(&self.node_states);

            move |event| {
                let payload = event.get_payload::<HeartbeatPayload>().unwrap();
                let latency = payload.get_beat_time().elapsed().ok()?.as_millis();
                log::debug!("Latency to node {} is {} ms", payload.node_id, latency);

                let mut states = node_states.lock();
                Self::insert_state(&mut states, payload.node_id, NodeInfo::alive(latency as u64));


                None
            }
        });
        if let Some(output) = &self.settings.output_file {
            pool.execute_at_fixed_rate(self.settings.interval(), self.settings.interval(), {
                let path = output.clone();
                let states = Arc::clone(&self.node_states);
                move || {
                    let states = states.lock();

                    if let Err(e) = write_json_pretty(&path, &*states) {
                        log::error!("Failed to write output states to file: {}", e)
                    }
                }
            });
        }

        Ok(())
    }

    fn boxed(self) -> Box<dyn Module> {
        Box::new(self)
    }

    fn tick(&mut self, server: &mut VentedServer, _: &mut ScheduledThreadPool) -> VentedResult<()> {
        if self.last_tick.elapsed() > self.settings.interval() {
            for node in server.nodes() {
                if let Err(e) = server.emit(node.id.clone(), Event::with_payload(HEARTBEAT_BEAT_EVENT, &HeartbeatPayload::now(server.node_id()))) {
                    log::debug!("Node {} is not reachable: {}", node.id, e);
                    let mut states = self.node_states.lock();
                    Self::insert_state(&mut states, node.id, NodeInfo::dead());
                }
            }
            self.last_tick = Instant::now();
        }

        Ok(())
    }
}

impl HeartbeatModule {
    fn insert_state(states: &mut HashMap<String, Vec<NodeInfo>>, id: String, state: NodeInfo) {
        lazy_static! {static ref MAX_RECORDS: usize = get_settings().modules.heartbeat.max_record_history;}
        if let Some(states) = states.get_mut(&id) {
            if states.len() > *MAX_RECORDS {
                states.remove(0);
            }
            states.push(state);
        } else {
            states.insert(id, vec![state]);
        }
    }
}