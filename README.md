# rbroadlink

A rust port of the [python-broadlink](https://github.com/mjg59/python-broadlink)
library.

## Client

This library includes an example client for communicating with broadlink devices.
The source for it is in `examples/rbroardlink-cli` and its usage is shown below:

```sh
rbroadlink 0.1.0
Nicholas Cioli <nicholascioli@gmail.com>, Wyatt Lindquist <git.wquist@gmail.com>
Command line arguments for the CLI

USAGE:
    rbroadlink-cli.exe <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    blast      Blasts an IR / RF code to the world
    connect    Connect a broadlink device to the network. Only tested on the RM3 Mini and the
               RM4 Pro
    help       Print this message or the help of the given subcommand(s)
    info       Get information about a broadlink device
    learn      Learn a code from a broadlink device on the network
    list       Lists available broadlink devices on the network
```
