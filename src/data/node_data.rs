use serde::{Serialize, Deserialize};
use vented::crypto::PublicKey;
use crate::utils::keys::{armor_public_key, extract_public_key};
use std::path::PathBuf;
use crate::utils::result::SnekcloudResult;
use std::fs;
use crate::utils::write_toml_pretty;

#[derive(Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub addresses: Vec<String>,
    public_key: String,
}

impl NodeData {
    pub fn new(id: String, public_key: PublicKey) -> Self {
        let public_key = armor_public_key(public_key);
        Self {
            id,
            addresses: Vec::with_capacity(0),
            public_key
        }
    }

    pub fn with_addresses(id: String, addresses: Vec<String>, public_key: PublicKey) -> Self {
        let public_key = armor_public_key(public_key);
        Self {
            id,
            addresses,
            public_key
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