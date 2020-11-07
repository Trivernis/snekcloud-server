use crate::utils::result::{SnekcloudResult, SnekcloudError};
use serde::{Serialize, Deserialize};
use crate::utils::{get_node_id, write_toml_pretty};
use std::fs;
use std::path::{Path, PathBuf};
use config::File;


const CONFIG_DIR: &str = "config/";
const DEFAULT_CONFIG: &str = "config/00_default.toml";
const GLOB_CONFIG: &str = "config/*.toml";
const ENV_PREFIX : &str = "SNEKCLOUD";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub listen_addresses: Vec<String>,
    pub node_id: String,
    pub private_key: PathBuf,
    pub node_data_dir: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            listen_addresses: vec!["127.0.0.1:22222".to_string()],
            node_id: get_node_id(),
            private_key: PathBuf::from("node_key"),
            node_data_dir: PathBuf::from("nodes"),
        }
    }
}

pub fn get_settings() -> SnekcloudResult<Settings> {
    if !Path::new(CONFIG_DIR).exists() {
        fs::create_dir(CONFIG_DIR)?;
    }
    write_toml_pretty(&PathBuf::from(DEFAULT_CONFIG), &Settings::default())?;

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name(DEFAULT_CONFIG))?
        .merge(glob::glob(GLOB_CONFIG)?.map(|path| File::from(path.unwrap()))
            .collect::<Vec<_>>())?
        .merge(config::Environment::with_prefix(ENV_PREFIX))?;


    settings.try_into().map_err(SnekcloudError::from)
}
