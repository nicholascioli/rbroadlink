# rbroadlink

A rust port of the [python-broadlink](https://github.com/mjg59/python-broadlink)
library.

## Devices Tested

The following devices have been tested and found to work with this library:

Model Code | Device Name | Manufacturer | Type
-----------|-------------|--------------|-----
0x649B | RM4 Pro | Broadlink | `Remote`

## Setup

Before a device can be used, it must be connected to a network. Refer to [this link](https://github.com/mjg59/python-broadlink#setup)
on how to get the device into AP mode, connect to its network (e.g. Broadlink_Device_Wifi), and
then run the following code:

```rust
use rbroadlink::Device;
use rbroadlink::network::WirelessConnection;

// Construct the network information
let network_info = WirelessConnection::WPA2(
    "SSID Here",
    "Password here",
);

// Connect the device to the specified network
Device::connect_to_network(&network_info)
    .expect("Could not connect the device to the network!");
```

You can also use the included cli to do so:

```sh
# Pass the password directly
cargo run --example rbroadlink-cli --features rbroadlink-cli -- connect wpa2 "SSID Here" "Password here"

# Prompt for the password safely
cargo run --example rbroadlink-cli --features rbroadlink-cli -- connect -p wpa2 "SSID Here"
```

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

// Use a remote-specific method to echo a learned IR code.
let code = remote_device.learn_ir()
    .expect("Could not learn IR code!");
remote_device.send_code(&code)
    .expect("Could not send code!");
```

## Examples

There are a few examples of this library present in the `examples` folder. Refer to
the [examples folder README](examples/README.md) for more info.
