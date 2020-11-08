use crate::modules::Module;
use vented::result::VentedResult;
use vented::server::VentedServer;
use crate::utils::result::SnekcloudResult;
use scheduled_thread_pool::ScheduledThreadPool;
use vented::server::server_events::{ NodeListPayload};
use vented::server::data::Node;
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::{HashMap};
use std::sync::atomic::{AtomicBool, Ordering};
use vented::crypto::PublicKey;
use std::time::{Instant, Duration, UNIX_EPOCH};
use crate::utils::settings::get_settings;
use crate::data::node_data::NodeData;
use std::path::PathBuf;
use crate::modules::nodes_refresh::settings::NodesRefreshSettings;

pub  mod settings;

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

    fn init(&mut self, server: &mut VentedServer, pool: &mut ScheduledThreadPool) -> SnekcloudResult<()> {
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
                        nodes.insert(node.id.clone(), Node {
                            id: node.id,
                            trusted: false,
                            public_key: PublicKey::from(node.public_key),
                            address: node.address,
                        });
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
                    nodes.lock().values().cloned().map(|node| {
                        if let Some(address) = node.address {
                            NodeData::with_addresses(node.id, vec![address], node.public_key)
                        } else {
                            NodeData::new(node.id, node.public_key)
                        }
                    }).for_each(|data| {
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

    fn boxed(self) -> Box<dyn Module> {
        Box::new(self)
    }

    fn tick(&mut self, server: &mut VentedServer, _: &mut ScheduledThreadPool) -> VentedResult<()> {
        if self.last_request.elapsed() > self.settings.update_interval() {
            if let Err(e) = server.request_node_list() {
                log::debug!("Failed to refresh node list: {}", e);
            }
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
            update_required: Arc::new(AtomicBool::new(false))
        }
    }
}