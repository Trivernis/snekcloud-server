use vented::server::VentedServer;
use crate::utils::result::SnekcloudResult;
use vented::crypto::SecretKey;
use vented::server::data::Node;
use vented::WaitGroup;
use crate::modules::Module;
use scheduled_thread_pool::ScheduledThreadPool;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

const SERVER_TICK_RATE_MS: u64 = 10;

pub struct SnekcloudServer {
    inner: VentedServer,
    listen_addresses: Vec<String>,
    listeners: Vec<WaitGroup>,
    module_pool: ScheduledThreadPool,
    modules: HashMap<String, Box<dyn Module>>
}

impl SnekcloudServer {
    /// Creates a new snekcloud server with the provided keys and number of threads
    pub fn new(id: String, private_key: SecretKey, keys: Vec<Node>, num_threads: usize) -> Self {
        Self {
            inner: VentedServer::new(id, private_key, keys, num_threads),
            listen_addresses: Vec::new(),
            listeners: Vec::new(),
            module_pool: ScheduledThreadPool::with_name("modules", num_threads),
            modules: HashMap::new()
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
        let sleep_duration = Duration::from_millis(SERVER_TICK_RATE_MS);
        loop {
            let start = Instant::now();
            for (name, module) in &mut self.modules {
                if let Err(e) = module.tick(&mut self.inner, &mut self.module_pool) {
                    log::error!("Error when ticking module {}: {}", name, e);
                }
            }
            let elapsed = start.elapsed();
            if elapsed < sleep_duration {
                thread::sleep( sleep_duration - elapsed);
            } else {
                log::warn!("Can't keep up. Last tick took {} ms", elapsed.as_millis())
            }
        }
    }

    /// Registers a module on the server
    pub fn register_module(&mut self, mut module: impl Module) -> SnekcloudResult<()> {
        module.init(&mut self.inner, &mut self.module_pool)?;
        self.modules.insert(module.name(), module.boxed());

        Ok(())
    }
}