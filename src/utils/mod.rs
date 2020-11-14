/*
 * snekcloud node based network
 * Copyright (C) 2020 trivernis
 * See LICENSE for more information
 */

use crate::utils::result::SnekcloudResult;
use rand::RngCore;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

pub mod keys;
pub mod logging;
pub mod result;
pub mod settings;

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
    serializer.pretty_array(true);
    value.serialize(&mut serializer)?;
    fs::write(path, buf_str.as_bytes())?;

    Ok(())
}

pub fn write_json_pretty<T: Serialize>(path: &PathBuf, value: &T) -> SnekcloudResult<()> {
    let string_value = serde_json::to_string_pretty(value)?;
    fs::write(path, string_value.as_bytes())?;

    Ok(())
}

pub fn validate_node_id(name: &str) -> bool {
    lazy_static! {
        static ref NODE_REGEX: Regex = Regex::new(r"^\S{1,32}$").expect("Failed to compile regex");
    }

    !name.eq_ignore_ascii_case("local") && NODE_REGEX.is_match(name)
}
