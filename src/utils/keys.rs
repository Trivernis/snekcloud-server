use crate::data::node_data::NodeData;
use crate::utils::result::{SnekcloudError, SnekcloudResult};
use crate::utils::settings::get_settings;
use crate::utils::validate_node_id;
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use vented::server::data::Node;
use vented::stream::{PublicKey, SecretKey};

const PRIVATE_KEY_HEADER_LINE: &str = "---BEGIN-SNEKCLOUD-PRIVATE-KEY---\n";
const PRIVATE_KEY_FOOTER_LINE: &str = "\n---END-SNEKCLOUD-PRIVATE-KEY---";

const PUBLIC_KEY_HEADER_LINE: &str = "---BEGIN-SNEKCLOUD-PUBLIC-KEY---\n";
const PUBLIC_KEY_FOOTER_LINE: &str = "\n---END-SNEKCLOUD-PUBLIC-KEY---";

/// Reads a folder of node public keys
pub fn read_node_keys(path: &PathBuf) -> SnekcloudResult<Vec<Node>> {
    if !Path::new(path).exists() {
        create_dir(path)?;
    }
    let trusted_nodes = get_settings().trusted_nodes;
    let own_id = get_settings().node_id;

    let content = glob::glob(format!("{}/*.toml", path.to_string_lossy()).as_str())?
        .filter_map(|path| {
            let path = path.ok()?;
            if path
                .file_name()?
                .to_string_lossy()
                .eq_ignore_ascii_case("local")
            {
                return None;
            }
            let data = NodeData::from_file(path).ok()?;

            Some(Node {
                public_key: data.public_key(),
                addresses: data.addresses,
                trusted: trusted_nodes.contains(&data.id),
                id: data.id,
            })
        })
        .filter(|node| validate_node_id(&node.id) && node.id != own_id)
        .collect();

    Ok(content)
}

/// Reads the private key from a file
#[inline]
pub fn extract_private_key(content: &str) -> SnekcloudResult<SecretKey> {
    let bytes = extract_key(content, PRIVATE_KEY_HEADER_LINE, PRIVATE_KEY_FOOTER_LINE)?;

    Ok(SecretKey::from(bytes))
}

/// Reads the public key from a file
#[inline]
pub fn extract_public_key(content: &str) -> SnekcloudResult<PublicKey> {
    let bytes = extract_key(content, PUBLIC_KEY_HEADER_LINE, PUBLIC_KEY_FOOTER_LINE)?;

    Ok(PublicKey::from(bytes))
}

/// Extracts a base64 encoded key between the prefix and suffix
fn extract_key(content: &str, prefix: &str, suffix: &str) -> SnekcloudResult<[u8; 32]> {
    if !content.starts_with(prefix) || !content.ends_with(suffix) {}
    let mut content = content.trim_start_matches(prefix);
    content = content.trim_end_matches(suffix);

    let key = base64::decode(content)?;
    if key.len() != 32 {
        return Err(SnekcloudError::InvalidKey);
    }
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&key[..]);

    Ok(key_bytes)
}

/// Encodes and encases the public key for text representation
#[inline]
pub fn armor_public_key(key: PublicKey) -> String {
    armor_key(
        key.to_bytes(),
        PUBLIC_KEY_HEADER_LINE,
        PUBLIC_KEY_FOOTER_LINE,
    )
}

/// Encodes and encases the secret key for text representation
#[inline]
pub fn armor_private_key(key: SecretKey) -> String {
    armor_key(
        key.to_bytes(),
        PRIVATE_KEY_HEADER_LINE,
        PRIVATE_KEY_FOOTER_LINE,
    )
}

/// Returns an armored key
#[inline]
fn armor_key(key: [u8; 32], prefix: &str, suffix: &str) -> String {
    format!("{}{}{}", prefix, base64::encode(key), suffix)
}

/// Generates a new private key
#[inline]
pub fn generate_private_key() -> SecretKey {
    SecretKey::generate(&mut rand::thread_rng())
}
