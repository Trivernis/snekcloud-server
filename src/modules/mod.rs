use vented::server::VentedServer;
use crate::utils::result::SnekcloudResult;
use scheduled_thread_pool::ScheduledThreadPool;
use vented::result::VentedResult;

pub mod heartbeat;

pub trait Module {
    fn name(&self) -> String;
    fn init(&mut self, server: &mut VentedServer, pool: &mut ScheduledThreadPool) -> SnekcloudResult<()>;
    fn boxed(self) -> Box<dyn Module>;
    fn tick(&mut self, server: &mut VentedServer, pool: &mut ScheduledThreadPool) -> VentedResult<()>;
}