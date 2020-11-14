/*
 * snekcloud node based network
 * Copyright (C) 2020 trivernis
 * See LICENSE for more information
 */

use crate::modules::Module;
use crate::server::tick_context::{EventInvocation, RunContext};
use crate::utils::result::{SnekcloudError, SnekcloudResult};
use crate::utils::settings::get_settings;

use async_std::sync::{channel, Receiver};
use async_std::task;
use std::collections::HashMap;
use std::mem;
use vented::server::data::Node;
use vented::server::VentedServer;
use vented::stream::SecretKey;

pub mod tick_context;

pub struct SnekcloudServer {
    inner: VentedServer,
    listen_addresses: Vec<String>,
    modules: HashMap<String, Box<dyn Module + Send + Sync>>,
}

impl SnekcloudServer {
    /// Creates a new snekcloud server with the provided keys and number of threads
    pub fn new(id: String, private_key: SecretKey, keys: Vec<Node>) -> Self {
        Self {
            inner: VentedServer::new(id, private_key, keys, get_settings().timeouts()),
            listen_addresses: Vec::new(),
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

        let mut modules = mem::take(&mut self.modules).into_iter();
        let (tx, rx) = channel(10);
        let tick_context = RunContext::new(self.inner.node_id(), tx, self.inner.nodes_ref());

        while let Some((name, mut module)) = modules.next() {
            let tick_context = RunContext::clone(&tick_context);
            task::spawn(async move {
                if let Err(e) = module.run(RunContext::clone(&tick_context)).await {
                    log::error!("Error when ticking module {}: {}", name, e);
                }
            });
        }

        task::block_on(self.handle_invocations(rx));

        Ok(())
    }

    /// Handles invocations
    async fn handle_invocations(&self, rx: Receiver<EventInvocation>) {
        while let Ok(mut invocation) = rx.recv().await {
            let inner = self.inner.clone();
            task::spawn(async move {
                let result = task::block_on(inner.emit(invocation.target_node, invocation.event));
                invocation
                    .result
                    .result(result.map_err(SnekcloudError::from));
            });
        }
    }

    /// Registers a module on the server
    pub fn register_module(
        &mut self,
        mut module: impl Module + Send + Sync,
    ) -> SnekcloudResult<()> {
        module.init(&mut self.inner)?;
        self.modules.insert(module.name(), module.boxed());

        Ok(())
    }
}
