use std::path::PathBuf;
use vented::crypto::{SecretKey, PublicKey};
use std::fs;
use crate::utils::result::{SnekcloudResult, SnekcloudError};
use vented::server::data::Node;

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
            let key = read_public_key(&entry.path()).ok()?;

            let file_name =  entry.file_name();
            let file_name = file_name.to_string_lossy();
            let node_id = file_name.trim_end_matches(".pub");

            Some(Node {public_key: key, address: None, id: node_id.to_string()})
        }).collect();

    Ok(content)
}

/// Reads the private key from a file
pub fn read_private_key(filename: &PathBuf) -> SnekcloudResult<SecretKey> {
    let content = fs::read_to_string(filename)?;

    let bytes = extract_key(content, PUBLIC_KEY_HEADER_LINE, PUBLIC_KEY_FOOTER_LINE)?;

    Ok(SecretKey::from(bytes))
}

/// Reads the public key from a file
pub fn read_public_key(filename: &PathBuf) -> SnekcloudResult<PublicKey> {
    let content = fs::read_to_string(filename)?;
    let bytes = extract_key(content, PUBLIC_KEY_HEADER_LINE, PUBLIC_KEY_FOOTER_LINE)?;

    Ok(PublicKey::from(bytes))
}

/// Extracts a base64 encoded key between the prefix and suffix
fn extract_key(content: String, prefix: &str, suffix: &str) -> SnekcloudResult<[u8; 32]> {
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