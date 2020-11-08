use crate::modules::heartbeat::settings::HeartbeatSettings;
use crate::modules::nodes_refresh::settings::NodesRefreshSettings;
use crate::utils::result::{SnekcloudError, SnekcloudResult};
use crate::utils::{get_node_id, write_toml_pretty};
use config::File;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_DIR: &str = "config/";
const DEFAULT_CONFIG: &str = "config/00_default.toml";
const GLOB_CONFIG: &str = "config/*.toml";
const ENV_PREFIX: &str = "SNEKCLOUD";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub listen_addresses: Vec<String>,
    pub node_id: String,
    pub private_key: PathBuf,
    pub node_data_dir: PathBuf,
    pub num_threads: usize,
    /// List of trusted nodes
    pub trusted_nodes: Vec<String>,
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
            private_key: PathBuf::from("node_key"),
            node_data_dir: PathBuf::from("nodes"),
            trusted_nodes: vec![],
            num_threads: num_cpus::get(),
            modules: ModuleSettings::default(),
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

    settings.try_into().map_err(SnekcloudError::from)
}
