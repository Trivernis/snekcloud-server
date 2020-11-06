use crate::utils::env::get_env_node_id;
use rand::RngCore;

pub mod result;
pub mod keys;
pub mod env;
pub mod logging;

pub fn get_node_id() -> String {
    if let Some(id) = get_env_node_id() {
        log::trace!("Using env node_id");
        id
    } else if let Ok(Some(address)) = mac_address::get_mac_address() {
        log::trace!("Using mac address as node_id");
        base64::encode(address.bytes())
    } else if let Ok(hostname) = hostname::get() {
        log::trace!("Using hostname as node_id");
        base64::encode(hostname.to_string_lossy().as_bytes())
    } else {
        log::trace!("Randomly generating node_id");
        let mut rng = rand::thread_rng();
        let mut id_raw = [0u8; 16];
        rng.fill_bytes(&mut id_raw);

        base64::encode(id_raw)
    }
}