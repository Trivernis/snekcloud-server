/*
 * snekcloud node based network
 * Copyright (C) 2020 trivernis
 * See LICENSE for more information
 */

use crate::server::tick_context::RunContext;
use crate::utils::result::SnekcloudResult;
use async_trait::async_trait;
use vented::server::VentedServer;

pub mod heartbeat;
pub mod nodes_refresh;

#[async_trait]
pub trait Module {
    fn name(&self) -> String;
    fn init(&mut self, server: &mut VentedServer) -> SnekcloudResult<()>;
    fn boxed(self) -> Box<dyn Module + Send + Sync>;
    async fn run(&mut self, context: RunContext) -> SnekcloudResult<()>;
}
