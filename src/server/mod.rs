use vented::server::VentedServer;
use crate::utils::result::SnekcloudResult;
use vented::crypto::SecretKey;
use vented::server::data::Node;
use vented::WaitGroup;
use crate::modules::Module;

pub struct SnekcloudServer {
    inner: VentedServer,
    listen_addresses: Vec<String>,
    listeners: Vec<WaitGroup>,
}

impl SnekcloudServer {
    /// Creates a new snekcloud server with the provided keys and number of threads
    pub fn new(id: String, private_key: SecretKey, keys: Vec<Node>, num_threads: usize) -> Self {
        Self {
            inner: VentedServer::new(id, private_key, keys, num_threads),
            listen_addresses: Vec::new(),
            listeners: Vec::new(),
        }
    }

    /// Adds an address the server should listen on
    pub fn add_listen_address(&mut self, address: String) {
        self.listen_addresses.push(address);
    }

    /// Starts listening on all addresses
    pub fn run(&mut self) -> SnekcloudResult<()> {
        for address in &self.listen_addresses {
            self.listeners.push(self.inner.listen(address.clone()))
        }

        Ok(())
    }

    /// Registers a module on the server
    pub fn register_module(&mut self, module: &mut impl Module) -> SnekcloudResult<()> {
        module.init(&mut self.inner)
    }
}