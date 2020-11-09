use crate::data::node_data::NodeData;
use crate::modules::nodes_refresh::settings::NodesRefreshSettings;
use crate::modules::Module;
use crate::server::tick_context::TickContext;
use crate::utils::result::SnekcloudResult;
use crate::utils::settings::get_settings;
use parking_lot::Mutex;
use scheduled_thread_pool::ScheduledThreadPool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};
use vented::event::Event;
use vented::server::data::Node;
use vented::server::server_events::{NodeListPayload, NODE_LIST_REQUEST_EVENT};
use vented::server::VentedServer;
use vented::stream::PublicKey;

pub mod settings;

pub struct NodesRefreshModule {
    nodes: Arc<Mutex<HashMap<String, Node>>>,
    update_required: Arc<AtomicBool>,
    last_request: Instant,
    settings: NodesRefreshSettings,
}

impl Module for NodesRefreshModule {
    fn name(&self) -> String {
        "node_list_refresh".to_string()
    }

    fn init(
        &mut self,
        server: &mut VentedServer,
        pool: &mut ScheduledThreadPool,
    ) -> SnekcloudResult<()> {
        {
            let mut node_list = self.nodes.lock();
            for node in server.nodes() {
                node_list.insert(node.id.clone(), node);
            }
        }
        server.on("conn:node_list", {
            let nodes = Arc::clone(&self.nodes);
            let update_required = Arc::clone(&self.update_required);

            move |event| {
                let mut nodes = nodes.lock();
                let mut new_nodes = false;

                for node in event.get_payload::<NodeListPayload>().ok()?.nodes {
                    if !nodes.contains_key(&node.id) {
                        nodes.insert(
                            node.id.clone(),
                            Node {
                                id: node.id,
                                trusted: false,
                                public_key: PublicKey::from(node.public_key),
                                addresses: node.addresses,
                            },
                        );
                        new_nodes = true;
                    }
                }

                if new_nodes {
                    update_required.store(true, Ordering::Relaxed)
                }
                None
            }
        });
        pool.execute_at_fixed_rate(Duration::from_secs(10), self.settings.update_interval(), {
            let nodes = Arc::clone(&self.nodes);
            let update_required = Arc::clone(&self.update_required);

            move || {
                if update_required.load(Ordering::Relaxed) {
                    let nodes_folder = get_settings().node_data_dir;
                    nodes
                        .lock()
                        .values()
                        .cloned()
                        .map(|node| {
                            NodeData::with_addresses(node.id, node.addresses, node.public_key)
                        })
                        .for_each(|data| {
                            let mut path = nodes_folder.clone();
                            path.push(PathBuf::from(format!("{}.toml", data.id)));

                            if let Err(e) = data.write_to_file(path) {
                                log::error!("Failed to write updated node data: {}", e);
                            }
                        });
                }
            }
        });

        Ok(())
    }

    fn boxed(self) -> Box<dyn Module + Send + Sync> {
        Box::new(self)
    }

    fn tick(
        &mut self,
        mut context: TickContext,
        _: &mut ScheduledThreadPool,
    ) -> SnekcloudResult<()> {
        if self.last_request.elapsed() > self.settings.update_interval() {
            context
                .living_nodes()
                .iter()
                .filter(|node| node.trusted)
                .for_each(|node| {
                    context.emit(node.id.clone(), Event::new(NODE_LIST_REQUEST_EVENT));
                });
            self.last_request = Instant::now();
        }

        Ok(())
    }
}

impl NodesRefreshModule {
    pub fn new() -> Self {
        let null_time = Instant::now() - UNIX_EPOCH.elapsed().unwrap();
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
            settings: get_settings().modules.nodes_refresh,
            last_request: null_time,
            update_required: Arc::new(AtomicBool::new(false)),
        }
    }
}
