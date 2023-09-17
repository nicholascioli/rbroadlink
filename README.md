# rbroadlink
[![Crates.io](https://img.shields.io/crates/v/rbroadlink.svg)](https://crates.io/crates/rbroadlink)
[![Crates.io](https://img.shields.io/crates/l/rbroadlink.svg)](https://crates.io/crates/rbroadlink)

A rust port of the [python-broadlink](https://github.com/mjg59/python-broadlink)
library.

## Devices Tested

The following devices have been tested and found to work with this library:

Model Code | Device Name | Manufacturer | Type
-----------|-------------|--------------|-----
0x649B | RM4 Pro | Broadlink | `Remote`
0x4E2A | [Ande Jupiter+](https://www.myande.pl/service/seria-jupiter-plus/) | ANG Klimatyzacja Sp. z o.o. | `Hvac`

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
cargo run --example rbroadlink-cli -- connect wpa2 "SSID Here" "Password here"

# Prompt for the password safely
cargo run --example rbroadlink-cli -- connect -p wpa2 "SSID Here"
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

## HVAC

Starting from version *0.4.0* of this library the HVAC/Air Conditioners support was added.
Supported devices are broadlink device type `0x4E2A`.

Although it was tested on one specific unit, it is very likely that it should work with more similar devices.
Those air conditioners are usually controlled with
[AC Freedom](https://play.google.com/store/apps/details?id=com.broadlink.acfreedom) application.

If you have such device connected to the _AC Freedom_ application, then it is surely in a "locked" state
(cannot be controlled using this library).

You can control it either from _AC Freedom_ or this library (not both). If you decide to use *rbroadlink*, then you
need to delete the device from _AC Freedom_ cloud, then reset the WiFi dongle and re-configure WiFi parameters again.

You can also head to this post for details:
https://github.com/liaan/broadlink_ac_mqtt/issues/76#issuecomment-884763601

Probably configuring the WiFi parameters using this library/rbroadlink-cli should also work (refer to the _Setup_ section above).

### A sample snippet for setting target temperature setpoint:
```rust
use rbroadlink::Device;

// Assuming that you have a valid device in `device`...
let hvac_device = match device {
    Device::Hvac { hvac } => hvac,
    _ => return Err("Not a HVAC device!"),
};

// First obtain current state/parameters of the device:
let mut state = hvac_device.get_state().expect("Cannot obtain current state");
println!("Current state: {:?}", state);

// Print current temperature and try to set a new setpoint (degree Celsius)
println!("Target temp: {:.1}", state.get_target_temp());
if let Err(e) = state.set_target_temp(22.0) {
    println!("Error setting temperature: {}", e);
}

// Request to set a new state (with new temperature)
hvac_device.set_state(&mut state);
```

## Examples

There are a few examples of this library present in the `examples` folder. Refer to
the [examples folder README](examples/README.md) for more info.
