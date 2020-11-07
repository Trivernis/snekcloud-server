use rand::RngCore;
use std::path::PathBuf;
use crate::utils::result::SnekcloudResult;
use serde::Serialize;
use std::fs;

pub mod result;
pub mod keys;
pub mod settings;
pub mod logging;



pub fn get_node_id() -> String {
    if let Ok(Some(address)) = mac_address::get_mac_address() {
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

/// Writes a pretty toml file to the given path
pub fn write_toml_pretty<T: Serialize>(path: &PathBuf, value: &T) -> SnekcloudResult<()> {
    let mut buf_str = String::new();
    let mut serializer = toml::Serializer::pretty(&mut buf_str);
    value.serialize(&mut serializer)?;
    fs::write(path, buf_str.as_bytes())?;

    Ok(())
}