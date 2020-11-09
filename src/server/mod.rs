use crate::modules::Module;
use crate::server::tick_context::TickContext;
use crate::utils::result::{SnekcloudError, SnekcloudResult};
use parking_lot::Mutex;
use scheduled_thread_pool::ScheduledThreadPool;
use std::cmp::max;
use std::collections::HashMap;
use std::mem;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;
use vented::server::data::Node;
use vented::server::VentedServer;
use vented::stream::SecretKey;
use vented::utils::result::VentedError;
use vented::WaitGroup;

pub mod tick_context;

const SERVER_TICK_RATE_MS: u64 = 10;

pub struct SnekcloudServer {
    inner: VentedServer,
    listen_addresses: Vec<String>,
    listeners: Vec<WaitGroup>,
    module_pool: HashMap<String, Arc<Mutex<ScheduledThreadPool>>>,
    modules: HashMap<String, Box<dyn Module + Send + Sync>>,
}

impl SnekcloudServer {
    /// Creates a new snekcloud server with the provided keys and number of threads
    pub fn new(id: String, private_key: SecretKey, keys: Vec<Node>, num_threads: usize) -> Self {
        let num_threads = max(num_threads, keys.len());
        Self {
            inner: VentedServer::new(id, private_key, keys, num_threads * 2, num_threads * 10),
            listen_addresses: Vec::new(),
            listeners: Vec::new(),
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
            self.listeners.push(self.inner.listen(address.clone()))
        }

        let modules = mem::take(&mut self.modules);
        let (tx, rx) = channel();
        let tick_context = TickContext::new(self.inner.node_id(), tx, self.inner.nodes_ref());
        let node_count = self.inner.nodes().len();

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
        let invocation_pool = ScheduledThreadPool::new(node_count);
        for invocation in rx {
            let mut future = self
                .inner
                .emit(invocation.target_node.clone(), invocation.event);
            let mut invocation_result = invocation.result;
            let node_id = invocation.target_node;

            invocation_pool.execute(move || {
                let result = future.get_value_with_timeout(Duration::from_secs(60));

                if let Some(result) = result {
                    invocation_result.result(result.map_err(SnekcloudError::from));
                } else {
                    log::error!("Failed to send event: Timeout after 5s");
                    invocation_result.reject(SnekcloudError::Vented(VentedError::UnreachableNode(
                        node_id,
                    )));
                }
            });
        }

        Ok(())
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
