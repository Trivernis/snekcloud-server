# Snekcloud

This repository contains the snekcloud server implementation.

## Usage

```
USAGE:
    snekcloud-server [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    generate-key       Generates a new private key
    help               Prints this message or the help of the given subcommand(s)
    write-info-file    
```

When run without a subcommand the server executes normally.


## Configuration

Basic server configuration is done in .env files.

|Variable | Description | Default Value |
|--------|----------|---------------|
| SNEKCLOUD_NODES_DIR | Directory containing the .toml files for the network nodes| nodes |
| SNEKCLOUD_PRIVATE_KEY | Path of the private key (generated with generate-key) | node_key |
| SNEKCLOUD_LISTEN_ADDRESS | The address the server listens on | 127.0.0.1:22222 |
| SNEKCLOUD_NODE_ID | The NodeID of the instance | None |

The NodeID is a parameter that can either be set manually or is generated
from the mac-address or hostname depending on what is available.


## License

This project is licensed under [GNU General Public License 3](https://github.com/Trivernis/snekcloud-server/blob/main/LICENSE).