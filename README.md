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

The configuration for the server has to be done in the config directory.
This directory will always contain the default configuration `default.toml` and will
load additional files with the same ending.

## License

This project is licensed under [GNU General Public License 3](https://github.com/Trivernis/snekcloud-server/blob/main/LICENSE).