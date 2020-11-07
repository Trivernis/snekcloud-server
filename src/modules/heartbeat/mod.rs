use crate::modules::Module;
use vented::server::VentedServer;
use crate::utils::result::SnekcloudResult;
use scheduled_thread_pool::ScheduledThreadPool;
use vented::result::{VentedResult};
use crate::modules::heartbeat::payloads::HeartbeatPayload;
use std::time::{Instant, Duration};
use vented::event::Event;

mod payloads;
const HEARTBEAT_BEAT_EVENT: &str = "heartbeat:beat";
const HEARTBEAT_RATE_SECONDS: u64 = 10;

pub struct HeartbeatModule {
    last_tick: Instant,
}

impl HeartbeatModule {
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now()
        }
    }
}

impl Module for HeartbeatModule {
    fn name(&self) -> String {
        "HeartbeatModule".to_string()
    }

    fn init(&mut self, server: &mut VentedServer, _: &mut ScheduledThreadPool) -> SnekcloudResult<()> {
        server.on(HEARTBEAT_BEAT_EVENT, |event| {
            let payload = event.get_payload::<HeartbeatPayload>().unwrap();
            log::debug!("Latency to node {} is {} ms", payload.node_id, payload.get_beat_time().elapsed().unwrap().as_millis());

            None
        });

        Ok(())
    }

    fn boxed(self) -> Box<dyn Module> {
        Box::new(self)
    }

    fn tick(&mut self, server: &mut VentedServer, _: &mut ScheduledThreadPool) -> VentedResult<()> {
        if self.last_tick.elapsed() > Duration::from_secs(HEARTBEAT_RATE_SECONDS) {
            for node in server.nodes() {
                if let Err(e) = server.emit(node.id.clone(), Event::with_payload(HEARTBEAT_BEAT_EVENT, &HeartbeatPayload::now(server.node_id()))) {
                    log::warn!("Node {} is not reachable: {}", node.id, e)
                }
            }
            self.last_tick = Instant::now();
        }

        Ok(())
    }
}