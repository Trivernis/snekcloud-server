use crate::server::SnekcloudServer;
use crate::utils::settings::{Settings, get_settings};
use crate::utils::keys::{extract_private_key, read_node_keys, generate_private_key, armor_private_key};
use std::path::PathBuf;
use crate::utils::result::SnekcloudResult;
use crate::utils::logging::init_logger;
use std::fs;
use structopt::StructOpt;
use vented::crypto::SecretKey;
use crate::data::node_data::NodeData;
use crate::modules::heartbeat::HeartbeatModule;

pub(crate) mod utils;
pub(crate) mod server;
pub(crate) mod data;
pub(crate) mod modules;

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
    let settings = get_settings()?;

    if let Some(command) = opt.sub_command {
        match command {
            SubCommand::GenerateKey(options) => {
                let key = generate_private_key();
                let string_content = armor_private_key(key);
                fs::write(options.output_file, string_content)?;
            },
            SubCommand::WriteInfoFile(options) => {
                let key = get_private_key(&settings)?;
                let data = NodeData::with_addresses(settings.node_id, settings.listen_addresses, key.public_key());
                data.write_to_file(options.output_file)?;
            }
        }
    } else {
        start_server(opt, &settings)?;
    }

    Ok(())
}

fn start_server(_options: Opt, settings: &Settings) -> SnekcloudResult<()> {
    let keys = read_node_keys(&settings.node_data_dir)?;
    let mut server = SnekcloudServer::new(settings.node_id.clone(), get_private_key(settings)?, keys, 8);

    for address in &settings.listen_addresses {
        server.add_listen_address(address.clone());
    }
    server.register_module(HeartbeatModule::new())?;
    server.run()?;

    Ok(())
}

fn get_private_key(settings: &Settings) -> SnekcloudResult<SecretKey> {
    extract_private_key(&fs::read_to_string(&settings.private_key)?)
}