use crate::server::SnekcloudServer;
use crate::utils::get_node_id;
use crate::utils::env::{get_private_key_path, get_key_file_storage, get_listen_address};
use crate::utils::keys::{extract_private_key, read_node_keys, generate_private_key, armor_private_key};
use std::path::PathBuf;
use crate::utils::result::SnekcloudResult;
use crate::utils::logging::init_logger;
use std::fs;
use structopt::StructOpt;
use vented::crypto::SecretKey;
use crate::data::node_data::NodeData;

pub(crate) mod utils;
pub(crate) mod server;
pub(crate) mod data;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(subcommand)]
    sub_command: Option<SubCommand>
}

#[derive(StructOpt, Debug)]
#[structopt()]
enum SubCommand {
    /// Generates a new private key
    GenerateKey(GenerateKeyOptions),

    WriteInfoFile(WriteInfoFileOptions)
}

#[derive(StructOpt, Debug)]
struct GenerateKeyOptions {
    /// The file the key is stored to
    #[structopt(parse(from_os_str))]
    output_file: PathBuf
}


#[derive(StructOpt, Debug)]
struct WriteInfoFileOptions {
    /// The file the info is stored to
    #[structopt(parse(from_os_str))]
    output_file: PathBuf
}

fn main() -> SnekcloudResult<()>{
    init_logger();
    let opt: Opt = Opt::from_args();
    if let Some(command) = opt.sub_command {
        match command {
            SubCommand::GenerateKey(options) => {
                let key = generate_private_key();
                let string_content = armor_private_key(key);
                fs::write(options.output_file, string_content)?;
            },
            SubCommand::WriteInfoFile(options) => {
                let key = get_private_key()?;
                let data = NodeData::with_address(get_node_id(), get_listen_address(), key.public_key());
                data.write_to_file(options.output_file)?;
            }
        }
    } else {
        start_server(opt)?;
    }

    Ok(())
}

fn start_server(_options: Opt) -> SnekcloudResult<()> {
    let keys = read_node_keys(&PathBuf::from(get_key_file_storage()))?;
    let mut server = SnekcloudServer::new(get_node_id(), get_private_key()?, keys, 8);
    server.add_listen_address(get_listen_address());
    server.run()?;

    Ok(())
}

fn get_private_key() -> SnekcloudResult<SecretKey> {
    extract_private_key(&fs::read_to_string(get_private_key_path())?)
}