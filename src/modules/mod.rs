use crate::server::tick_context::TickContext;
use crate::utils::result::SnekcloudResult;
use scheduled_thread_pool::ScheduledThreadPool;
use vented::server::VentedServer;

pub mod heartbeat;
pub mod nodes_refresh;

pub trait Module {
    fn name(&self) -> String;
    fn init(
        &mut self,
        server: &mut VentedServer,
        pool: &mut ScheduledThreadPool,
    ) -> SnekcloudResult<()>;
    fn boxed(self) -> Box<dyn Module + Send + Sync>;
    fn tick(&mut self, context: TickContext, pool: &mut ScheduledThreadPool)
        -> SnekcloudResult<()>;
}
