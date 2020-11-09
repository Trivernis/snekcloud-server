use crate::utils::keys::{armor_public_key, extract_public_key};
use crate::utils::result::SnekcloudResult;
use crate::utils::write_toml_pretty;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use vented::stream::PublicKey;

#[derive(Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub addresses: Vec<String>,
    public_key: String,
}

impl NodeData {
    pub fn with_addresses(id: String, addresses: Vec<String>, public_key: PublicKey) -> Self {
        let public_key = armor_public_key(public_key);
        Self {
            id,
            addresses,
            public_key,
        }
    }

    /// Creates the data structure from a given file
    pub fn from_file(path: PathBuf) -> SnekcloudResult<Self> {
        let content = fs::read_to_string(path)?;
        let result = toml::from_str(&content)?;

        Ok(result)
    }

    /// Writes the data to the given file
    pub fn write_to_file(&self, path: PathBuf) -> SnekcloudResult<()> {
        write_toml_pretty(&path, self)
    }

    pub fn public_key(&self) -> PublicKey {
        extract_public_key(&self.public_key).unwrap()
    }
}
