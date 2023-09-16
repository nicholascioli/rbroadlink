use packed_struct::prelude::PackedStruct;

use crate::traits::CommandTrait;

/// A message used to authenticate with a broadlink device on the network.
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "0x50")]
pub struct AuthenticationMessage {
    /// Device representative ID.
    /// Note: We use a dummy value here, but the OEM software uses the devices IMEI.
    #[packed_field(bytes = "0x04:0x14")]
    id: [u8; 16],

    /// Magic value 0 (always set to 1)
    #[packed_field(bytes = "0x1E")]
    magic0: u8,

    /// Magic value 1 (always set to 1)
    #[packed_field(bytes = "0x2D")]
    magic1: u8,

    /// The name of the device
    #[packed_field(bytes = "0x30:0x50")]
    name: [u8; 0x20],
}

/// The response to an authenticate request for a broadlink device on the network.
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "0x14")]
pub struct AuthenticationResponse {
    /// Device authentication ID.
    #[packed_field(bytes = "0x00:0x03")]
    pub id: u32,

    /// Device encryption key.
    #[packed_field(bytes = "0x04:0x13")]
    pub key: [u8; 16],
}

impl AuthenticationMessage {
    /// Construct a new AuthenticationMessage. Name should correspond to the name
    /// of the device, as presented by the device.
    pub fn new(name: &str) -> AuthenticationMessage {
        let mut fixed_name = [0u8; 0x20];

        // Copy over the name
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len();
        let max = if name_len > 0x20 { 0x20 } else { name_len };
        for i in 0..max {
            fixed_name[i] = name_bytes[i];
        }

        return AuthenticationMessage {
            id: [0x31u8; 16],
            magic0: 0x1,
            magic1: 0x1,

            name: fixed_name,
        };
    }
}

impl CommandTrait for AuthenticationMessage {
    fn packet_type() -> u16 {
        return 0x0065;
    }
}
