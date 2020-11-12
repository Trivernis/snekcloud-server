use crate::modules::Module;
use crate::server::tick_context::{EventInvocation, TickContext};
use crate::utils::result::{SnekcloudError, SnekcloudResult};
use crate::utils::settings::get_settings;
use parking_lot::Mutex;
use scheduled_thread_pool::ScheduledThreadPool;

use async_std::task;
use std::collections::HashMap;
use std::mem;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;
use vented::server::data::Node;
use vented::server::VentedServer;
use vented::stream::SecretKey;

pub mod tick_context;

const SERVER_TICK_RATE_MS: u64 = 10;

pub struct SnekcloudServer {
    inner: VentedServer,
    listen_addresses: Vec<String>,
    module_pool: HashMap<String, Arc<Mutex<ScheduledThreadPool>>>,
    modules: HashMap<String, Box<dyn Module + Send + Sync>>,
}

impl SnekcloudServer {
    /// Creates a new snekcloud server with the provided keys and number of threads
    pub fn new(id: String, private_key: SecretKey, keys: Vec<Node>) -> Self {
        Self {
            inner: VentedServer::new(id, private_key, keys, get_settings().timeouts()),
            listen_addresses: Vec::new(),
            module_pool: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    /// Adds an address the server should listen on
    pub fn add_listen_address(&mut self, address: String) {
        self.listen_addresses.push(address);
    }

    /// Starts listening on all addresses and runs the module tick loop
    pub fn run(&mut self) -> SnekcloudResult<()> {
        for address in &self.listen_addresses {
            self.inner.listen(address.clone())
        }

        let modules = mem::take(&mut self.modules);
        let (tx, rx) = channel();
        let tick_context = TickContext::new(self.inner.node_id(), tx, self.inner.nodes_ref());

        for (name, mut module) in modules {
            self.module_pool
                .get(&name)
                .unwrap()
                .lock()
                .execute_at_fixed_rate(
                    Duration::from_millis(SERVER_TICK_RATE_MS),
                    Duration::from_millis(SERVER_TICK_RATE_MS),
                    {
                        let mut module_pool = ScheduledThreadPool::new(1);
                        let tick_context = TickContext::clone(&tick_context);
                        move || {
                            if let Err(e) =
                                module.tick(TickContext::clone(&tick_context), &mut module_pool)
                            {
                                log::error!("Error when ticking module {}: {}", name, e);
                            }
                        }
                    },
                );
        }

        task::block_on(self.handle_invocations(rx));

        Ok(())
    }

    /// Handles invocations
    async fn handle_invocations(&self, rx: Receiver<EventInvocation>) {
        for mut invocation in rx {
            let result = self
                .inner
                .emit(invocation.target_node.clone(), invocation.event)
                .await;
            invocation
                .result
                .result(result.map_err(SnekcloudError::from));
        }
    }

    /// Registers a module on the server
    pub fn register_module(
        &mut self,
        mut module: impl Module + Send + Sync,
    ) -> SnekcloudResult<()> {
        let module_pool = Arc::new(Mutex::new(ScheduledThreadPool::new(2)));

        module.init(&mut self.inner, &mut module_pool.lock())?;
        self.module_pool.insert(module.name(), module_pool);
        self.modules.insert(module.name(), module.boxed());

        Ok(())
    }
}
