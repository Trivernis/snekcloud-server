use crate::modules::Module;
use crate::server::tick_context::TickContext;
use crate::utils::result::{SnekcloudError, SnekcloudResult};
use parking_lot::Mutex;
use scheduled_thread_pool::ScheduledThreadPool;
use std::collections::HashMap;
use std::mem;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;
use vented::crypto::SecretKey;
use vented::server::data::Node;
use vented::server::VentedServer;
use vented::WaitGroup;

pub mod tick_context;

const SERVER_TICK_RATE_MS: u64 = 10;

pub struct SnekcloudServer {
    inner: VentedServer,
    listen_addresses: Vec<String>,
    listeners: Vec<WaitGroup>,
    module_pool: Arc<Mutex<ScheduledThreadPool>>,
    modules: HashMap<String, Box<dyn Module + Send + Sync>>,
}

impl SnekcloudServer {
    /// Creates a new snekcloud server with the provided keys and number of threads
    pub fn new(id: String, private_key: SecretKey, keys: Vec<Node>, num_threads: usize) -> Self {
        Self {
            inner: VentedServer::new(id, private_key, keys, num_threads),
            listen_addresses: Vec::new(),
            listeners: Vec::new(),
            module_pool: Arc::new(Mutex::new(ScheduledThreadPool::with_name(
                "modules",
                num_threads,
            ))),
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
        let module_pool = Arc::clone(&self.module_pool);
        let (tx, rx) = channel();
        let tick_context = TickContext::new(self.inner.node_id(), tx, self.inner.nodes_ref());

        for (name, mut module) in modules {
            module_pool.lock().execute_at_fixed_rate(
                Duration::from_millis(SERVER_TICK_RATE_MS),
                Duration::from_millis(SERVER_TICK_RATE_MS),
                {
                    let module_pool = Arc::clone(&module_pool);
                    let tick_context = TickContext::clone(&tick_context);
                    move || {
                        let mut module_pool = module_pool.lock();

                        if let Err(e) =
                            module.tick(TickContext::clone(&tick_context), &mut module_pool)
                        {
                            log::error!("Error when ticking module {}: {}", name, e);
                        }
                    }
                },
            );
        }
        for mut invocation in rx {
            match self.inner.emit(invocation.target_node, invocation.event) {
                Ok(_) => invocation.result.set_value(Arc::new(Ok(()))),
                Err(e) => invocation
                    .result
                    .set_value(Arc::new(Err(SnekcloudError::from(e)))),
            }
        }

        Ok(())
    }

    /// Registers a module on the server
    pub fn register_module(
        &mut self,
        mut module: impl Module + Send + Sync,
    ) -> SnekcloudResult<()> {
        let mut module_pool = self.module_pool.lock();

        module.init(&mut self.inner, &mut module_pool)?;
        self.modules.insert(module.name(), module.boxed());

        Ok(())
    }
}
