# rbroadlink

A rust port of the [python-broadlink](https://github.com/mjg59/python-broadlink)
library.

## Devices Tested

The following devices have been tested and found to work with this library:

Model Code | Device Name | Manufacturer | Type
-----------|-------------|--------------|-----
0x649B | RM4 Pro | Broadlink | `Remote`

## Usage

Devices can either be constructed from a known IP or by local discovery:

```rust
use std::net::Ipv4Addr;
use rbroadlink::Device;

// Create a device by IP
// Note: Devices only support Ipv4 addresses
let known_ip = Ipv4Addr::new(1, 2, 3, 4);
let device = Device::from_ip(known_ip, None)
    .expect("Could not connect to device!");

// You can also specify the local IP of the machine in the case of the device being
// on a different subnet.
let local_ip = Ipv4::new(9, 8, 7, 6);
let device_with_local_ip = Device::from_ip(known_ip, Some(local_ip))
    .expect("Could not connect to device!");

// You can also just enumerate all of the discovered devices, with an optional
// local ip as well.
let devices = Device::list(Some(local_ip))
    .expect("Could not enumerate devices!");
```

Once you have a valid device, you probably want to differentiate the kind of device
that you have. `Device` is a structured enum that contains different types of devices with
more specialized methods.

```rust
use rbroadlink::Device;

// Assuming that you have a valid device in `device`...
let remote_device = match device {
    Device::Remote { remote } => remote,
    _ => return Err("Not a remote!"),
};

// Use a remote-specific method to echo a learned code.
let code = remote_device.learn_ir()
    .expect("Could not learn code!");
remote_device.send_ir(&code)
    .expect("Could not send code!");
```

## Client

This library includes an example client for communicating with broadlink devices.
The source for it is in `examples/rbroardlink-cli` and its usage is shown below:

```sh
rbroadlink 0.1.0
Nicholas Cioli <nicholascioli@gmail.com>, Wyatt Lindquist <git.wquist@gmail.com>
Command line arguments for the CLI

USAGE:
    rbroadlink-cli <SUBCOMMAND>

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

You can run the client from this source by running the following:

```sh
cargo run --example rbroadlink-cli --release -- <OPTIONS GO HERE>
```
