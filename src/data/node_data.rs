use serde::{Serialize, Deserialize};
use vented::crypto::PublicKey;
use crate::utils::keys::{armor_public_key, extract_public_key};
use std::path::PathBuf;
use crate::utils::result::SnekcloudResult;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub address: Option<String>,
    public_key: String,
}

impl NodeData {
    pub fn with_address(id: String, address: String, public_key: PublicKey) -> Self {
        let public_key = armor_public_key(public_key);
        Self {
            id,
            address: Some(address),
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
        let content = toml::to_string(self)?;
        fs::write(path, content.as_bytes())?;

        Ok(())
    }

    pub fn public_key(&self) -> PublicKey {
        extract_public_key(&self.public_key).unwrap()
    }
}