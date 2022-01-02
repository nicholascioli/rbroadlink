use std::net::Ipv4Addr;

/// Represents a broadlink device core information.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// The IP address of this device.
    pub address: Ipv4Addr,

    /// The MAC address of this device.
    pub mac: [u8; 6],

    /// The model code of this device.
    pub model_code: u16,

    /// The friendly model type
    pub friendly_model: String,

    /// The friendly device type
    pub friendly_type: String,

    /// The name of the device.
    pub name: String,

    /// The lock status of the device.
    pub is_locked: bool,

    /// The authentication ID used for encrypted communication.
    pub auth_id: u32,

    /// The key used for encrypted communication
    pub key: [u8; 16],
}
