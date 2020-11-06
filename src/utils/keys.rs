use std::path::PathBuf;
use vented::crypto::{SecretKey, PublicKey};
use crate::utils::result::{SnekcloudResult, SnekcloudError};
use vented::server::data::Node;
use crate::data::node_data::NodeData;

const PRIVATE_KEY_HEADER_LINE: &str = "---BEGIN-SNEKCLOUD-PRIVATE-KEY---\n";
const PRIVATE_KEY_FOOTER_LINE: &str = "\n---END-SNEKCLOUD-PRIVATE-KEY---";

const PUBLIC_KEY_HEADER_LINE: &str = "---BEGIN-SNEKCLOUD-PUBLIC-KEY---\n";
const PUBLIC_KEY_FOOTER_LINE: &str = "\n---END-SNEKCLOUD-PUBLIC-KEY---";

/// Reads a folder of node public keys
pub fn read_node_keys(path: &PathBuf) -> SnekcloudResult<Vec<Node>> {
    let dir_content = path.read_dir()?;

    let content = dir_content
        .filter_map(|entry| {
            let entry = entry.ok()?;

            Some((entry.metadata().ok()?, entry))
        })
        .filter(|(meta, _)|meta.is_file())
        .filter_map(|(_, entry)|{
            let data = NodeData::from_file(entry.path()).ok()?;

            Some(Node {public_key: data.public_key(), address: data.address, id: data.id})
        }).collect();

    Ok(content)
}

/// Reads the private key from a file
pub fn extract_private_key(content: &str) -> SnekcloudResult<SecretKey> {
    let bytes = extract_key(content, PRIVATE_KEY_HEADER_LINE, PRIVATE_KEY_FOOTER_LINE)?;

    Ok(SecretKey::from(bytes))
}

/// Reads the public key from a file
pub fn extract_public_key(content: &str) -> SnekcloudResult<PublicKey> {
    let bytes = extract_key(content, PUBLIC_KEY_HEADER_LINE, PUBLIC_KEY_FOOTER_LINE)?;

    Ok(PublicKey::from(bytes))
}

/// Extracts a base64 encoded key between the prefix and suffix
fn extract_key(content: &str, prefix: &str, suffix: &str) -> SnekcloudResult<[u8; 32]> {
    let mut content = content.strip_prefix(prefix).ok_or(SnekcloudError::InvalidKey)?;
    content = content.strip_suffix(suffix).ok_or(SnekcloudError::InvalidKey)?;

    let key = base64::decode(content)?;
    if key.len() != 32 {
        return Err(SnekcloudError::InvalidKey);
    }
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&key[..]);

    Ok(key_bytes)
}

/// Encodes and encases the public key for text representation
pub fn armor_public_key(key: PublicKey) -> String {
    armor_key(key.to_bytes(), PUBLIC_KEY_HEADER_LINE, PUBLIC_KEY_FOOTER_LINE)
}

/// Encodes and encases the secret key for text representation
pub fn armor_private_key(key: SecretKey) -> String {
    armor_key(key.to_bytes(), PRIVATE_KEY_HEADER_LINE, PRIVATE_KEY_FOOTER_LINE)
}

/// Returns an armored key
fn armor_key(key: [u8; 32], prefix: &str, suffix: &str) -> String {
    format!("{}{}{}", prefix, base64::encode(key), suffix)
}

/// Generates a new private key
pub fn generate_private_key() -> SecretKey {
    SecretKey::generate(&mut rand::thread_rng())
}