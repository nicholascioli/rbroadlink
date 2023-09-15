use std::{
    fmt,
    net::{
        IpAddr,
        Ipv4Addr,
        SocketAddr,
    },
    str::from_utf8,
};

use packed_struct::prelude::{ PackedStruct, PackedStructSlice };

use crate::{
    REMOTE_CODES,

    DeviceInfo,
    RemoteDevice,

    network::{
        AuthenticationMessage,
        AuthenticationResponse,
        CommandMessage,
        DiscoveryMessage,
        DiscoveryResponse,
        WirelessConnection,
        WirelessConnectionMessage,

        util::{
            local_ip_or,
            send_and_receive_many,
            send_and_receive_one,
        }
    },

    traits::{
        CommandTrait,
        DeviceTrait,
    },
};

/// A generic broadlink device.
pub enum Device {
    /// A device capable of transmitting IR / RF codes.
    Remote { remote: RemoteDevice },
}

/// Represents a generic device. See the different implementations for more specific info.
impl Device {
    /// Create a new device directly from an IP.
    pub fn from_ip(addr: Ipv4Addr, local_ip: Option<Ipv4Addr>) -> Result<Device, String> {
        // Grab the first non-loopback address
        let selected_ip = local_ip_or(local_ip)?;

        // Construct the discovery message
        let port = 42424;
        let discover = DiscoveryMessage::new(selected_ip, port, None)?;
        let msg = discover.pack().map_err(|e| format!("Could not pack DiscoveryMessage! {}", e))?;

        return Ok(
            send_and_receive_one(&msg, addr, Some(port), |bytes_received, bytes, addr| {
                return create_device_from_packet(addr, bytes_received, bytes);
            }).map_err(|e| format!("Could not communicate with specified device! {}", e))?
        );
    }

    /// List all devices in the current network. Optionally specify the local IP if on different subnets.
    pub fn list(ip: Option<Ipv4Addr>) -> Result<Vec<Device>, String> {
        // Grab the first non-loopback address
        let selected_ip = local_ip_or(ip)?;

        // Construct the discovery message
        let port = 42424;
        let discover = DiscoveryMessage::new(selected_ip, port, None)?;
        let msg = discover.pack().map_err(|e| format!("Could not pack DiscoveryMessage! {}", e))?;

        let results = send_and_receive_many(&msg, Ipv4Addr::BROADCAST, Some(port), |bytes_received, bytes, addr| {
            return Ok(
                create_device_from_packet(addr, bytes_received, &bytes)
                    .map_err(|e| format!("Could not create device from packet! {}", e))?
            )
        }).map_err(|e| format!("Could not send discovery message! {}", e))?;

        // Remove duplicates
        // TODO

        return Ok(results);
    }

    /// Authenticate a device. This is needed before any commands can be sent.
    ///
    /// Note: This is automatically called when constructing a device.
    pub fn authenticate(&mut self) -> Result<(), String> {
        let info = self.get_info();

        // Create the actual auth message
        let msg = AuthenticationMessage::new(&info.name);
        let packed = msg.pack().map_err(|e| format!("Could not pack Authentication message! {}", e))?;

        // Send the auth message command to the device
        let response = self.send_command::<AuthenticationMessage>(&packed)
            .map_err(|e| format!("Could not send authentication command! {}", e))?;

        // Unpack the response
        let auth = AuthenticationResponse::unpack_from_slice(&response)
            .map_err(|e| format!("Could not unpack auth response! {}", e))?;

        // Save the returned key and ID
        self.save_auth_pair(auth.id, auth.key);

        return Ok(());
    }

    /// Connects any found device to a specified network. Requires the host machine
    /// to connect to the device directly. Refer to -> <https://github.com/mjg59/python-broadlink#setup>
    pub fn connect_to_network(network: &WirelessConnection) -> Result<WirelessConnectionMessage, String> {
        let msg = network.to_message()
            .map_err(|e| format!("Could not create wireless connection message! {}", e))?;
        let packed = msg.pack()
            .map_err(|e| format!("Could not pack wireless connection message! {}", e))?;

        // We don't know the format of the response, so we just pass here.
        send_and_receive_one(&packed, Ipv4Addr::BROADCAST, None, |_, _, _| {
            return Ok(());
        }).map_err(|e| format!("Could not send connection message! {}", e))?;

        return Ok(msg);
    }

    /// Sends a raw command to a broadlink device.
    /// Note: Try to avoid using this method in favor of more specific methods (e.g. [Device::authenticate], etc.)
    pub fn send_command<T>(&self, payload: &[u8]) -> Result<Vec<u8>, String>
    where
        T: CommandTrait,
    {
        let info = self.get_info();

        // Construct the command.
        let cmd = CommandMessage::new::<T>(
            info.model_code,
            info.mac,
            info.auth_id,
        );

        // Pack the message with the payload
        let packed = cmd.pack_with_payload(&payload, &info.key)
            .map_err(|e| format!("Could not pack command with payload! {}", e))?;

        // Send the message to the device
        return send_and_receive_one(&packed, info.address, None, |_, bytes, _| {
            return CommandMessage::unpack_with_payload(bytes.to_vec(), &info.key);
        });
    }
}

// Delegate all device trait functions to the devices themselves
impl DeviceTrait for Device {
    /// Get the core information about a device.
    fn get_info(&self) -> DeviceInfo {
        return match self {
            Device::Remote { remote } => remote.info.clone(),
        };
    }

    /// Save the authentication information
    fn save_auth_pair(&mut self, id: u32, key: [u8; 16]) {
        return match self {
            Device::Remote { remote } => {
                remote.info.auth_id = id;
                remote.info.key = key;
            },
        };
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let info = self.get_info();

        write!(
            f,
            "{} [{} {:?}] (address = {}, mac = {}, locked? = {})",
            info.name,
            info.friendly_type,
            info.friendly_model,
            info.address,
            info.mac.iter().map(|x| format!("{:02X}", x)).collect::<Vec<String>>().join(":"),
            info.is_locked,
        )
    }
}

/// Creates a device from a received network packet.
fn create_device_from_packet(addr: SocketAddr, bytes_received: usize, bytes: &[u8]) -> Result<Device, String>
{
    // Make sure that we have the required amount of bytes
    if bytes_received != 128 {
        return Err("Received invalid response! Not enough data.".into());
    }

    // Short-circuit if the device is using an IPv6 address (should be impossible)
    let addr_ip = match addr.ip() {
        IpAddr::V4(a) => a,
        _ => return Err("Device has an IPv6 Address! This should be impossible...".into())
    };

    let response = DiscoveryResponse::unpack_from_slice(&bytes)
        .map_err(|e| format!("Could not unpack response from device! {}", e))?;

    // Decode the name
    let raw_name = response.name.clone();
    let name = from_utf8(&raw_name).map_err(|e| format!("Could not decode device name! {}", e))?;

    // Create the device conditionally based on the model code.
    let mut device = match &response.model_code {
        _ if REMOTE_CODES.contains_key(&response.model_code) => Device::Remote {
            remote: RemoteDevice::new(name, addr_ip, response)
        },
        _ => return Err(format!("Unknown device: {}", response.model_code)),
    };

    // Get the auth key for this device
    device.authenticate()
        .map_err(|e| format!("Could not authenticate device! {}", e))?;

    return Ok(device);
}
