const VAR_KEY_FILE_STORAGE: &str = "SNEKCLOUD_NODES_DIR";
const VAR_PRIVATE_KEY_PATH: &str = "SNEKCLOUD_PRIVATE_KEY";
const VAR_LISTEN_ADDRESS: &str = "SNEKCLOUD_LISTEN_ADDRESS";
const VAR_NODE_ID: &str = "SNEKCLOUD_NODE_ID";

pub fn get_key_file_storage() -> String {
    dotenv::var(VAR_KEY_FILE_STORAGE).unwrap_or("keys".to_string())
}

pub fn get_private_key_path() -> String {
    dotenv::var(VAR_PRIVATE_KEY_PATH).unwrap_or("node_key".to_string())
}

pub fn get_env_node_id() -> Option<String> {
    dotenv::var(VAR_NODE_ID).ok()
}

pub fn get_listen_address() -> String {
    dotenv::var(VAR_LISTEN_ADDRESS).unwrap_or("127.0.0.1:22222".to_string())
}