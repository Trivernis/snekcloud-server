/*
 * snekcloud node based network
 * Copyright (C) 2020 trivernis
 * See LICENSE for more information
 */

use crate::modules::heartbeat::settings::HeartbeatSettings;
use crate::modules::nodes_refresh::settings::NodesRefreshSettings;
use crate::utils::result::SnekcloudResult;
use crate::utils::{get_node_id, validate_node_id, write_toml_pretty};
use config::File;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use vented::server::data::ServerTimeouts;

const CONFIG_DIR: &str = "config/";
const DEFAULT_CONFIG: &str = "config/00_default.toml";
const GLOB_CONFIG: &str = "config/*.toml";
const ENV_PREFIX: &str = "SNEKCLOUD";

pub trait ValidateSettings {
    fn validate(&self);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub listen_addresses: Vec<String>,
    pub node_id: String,
    pub private_key: PathBuf,
    pub node_data_dir: PathBuf,
    pub trusted_nodes: Vec<String>,
    pub send_timeout_secs: u64,
    pub redirect_timeout_secs: u64,
    pub log_folder: PathBuf,
    // modules need to be last because it's a table
    pub modules: ModuleSettings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModuleSettings {
    pub heartbeat: HeartbeatSettings,
    pub nodes_refresh: NodesRefreshSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            listen_addresses: vec![],
            node_id: get_node_id(),
            private_key: PathBuf::from("private_key"),
            node_data_dir: PathBuf::from("nodes"),
            log_folder: PathBuf::from("logs"),
            trusted_nodes: vec![],
            send_timeout_secs: 5,
            redirect_timeout_secs: 20,
            modules: ModuleSettings::default(),
        }
    }
}

impl Settings {
    pub fn timeouts(&self) -> ServerTimeouts {
        ServerTimeouts {
            redirect_timeout: Duration::from_secs(self.redirect_timeout_secs),
            send_timeout: Duration::from_secs(self.send_timeout_secs),
        }
    }
}

impl Default for ModuleSettings {
    fn default() -> Self {
        Self {
            heartbeat: HeartbeatSettings::default(),
            nodes_refresh: NodesRefreshSettings::default(),
        }
    }
}

impl ValidateSettings for Settings {
    fn validate(&self) {
        if !self.private_key.exists() {
            panic!(format!("Private key {:?} does not exist", self.private_key));
        }
        if self.send_timeout_secs == 0 {
            panic!("Send timeout must be greater than 0");
        }
        if self.redirect_timeout_secs == 0 {
            panic!("Redirect timeout must be greater than 0");
        }
        if !validate_node_id(&self.node_id) {
            panic!(format!("Invalid NodeID {}", self.node_id));
        }
        self.modules.validate();
    }
}

impl ValidateSettings for ModuleSettings {
    fn validate(&self) {}
}

/// Returns the settings that are lazily retrieved at runtime
pub fn get_settings() -> Settings {
    lazy_static! {
        static ref SETTINGS: Settings = load_settings().expect("Failed to get settings");
    }

    SETTINGS.clone()
}

fn load_settings() -> SnekcloudResult<Settings> {
    if !Path::new(CONFIG_DIR).exists() {
        fs::create_dir(CONFIG_DIR)?;
    }
    write_toml_pretty(&PathBuf::from(DEFAULT_CONFIG), &Settings::default())?;

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name(DEFAULT_CONFIG))?
        .merge(
            glob::glob(GLOB_CONFIG)?
                .map(|path| File::from(path.unwrap()))
                .collect::<Vec<_>>(),
        )?
        .merge(config::Environment::with_prefix(ENV_PREFIX))?;

    let settings: Settings = settings.try_into()?;

    Ok(settings)
}
