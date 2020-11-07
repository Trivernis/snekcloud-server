use vented::server::VentedServer;
use crate::utils::result::SnekcloudResult;

pub trait Module {
    fn init(&mut self, server: &VentedServer) -> SnekcloudResult<()>;
}