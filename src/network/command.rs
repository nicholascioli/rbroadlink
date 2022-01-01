use aes::Aes128;
use block_modes::{ BlockMode, Cbc };
use block_modes::block_padding::ZeroPadding;
use packed_struct::prelude::{ PackedStruct, PackedStructSlice };
use rand::Rng;

use crate::{
    constants,
    network::util::{
        checksum,
        reverse_mac,
    },
    traits::CommandTrait,
};

/// Represents a block-based AES 128-bit encryption cipher.
pub type AesCbc = Cbc<Aes128, ZeroPadding>;

/// A message used to send commands to a broadlink device on the network.
#[derive(PackedStruct, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "0x38")]
pub struct CommandMessage {
    /// Magic header
    #[packed_field(bytes = "0x00:0x07")]
    magic_header: [u8; 0x08],

    /// The type of device receiving this message
    #[packed_field(bytes = "0x24:0x25")]
    device_type: u16,

    /// The type of packet being sent.
    #[packed_field(bytes = "0x26:0x27")]
    packet_type: u16,

    /// The message count.
    #[packed_field(bytes = "0x28:0x29")]
    count: u16,

    /// The mac address, reversed.
    #[packed_field(bytes = "0x2A:0x2F")]
    mac_reversed: [u8; 6],

    /// The device ID. Set to 0 when authenticating for the first time.
    #[packed_field(bytes = "0x30:0x33")]
    id: u32,

    /// The checksum of the entire command + payload
    #[packed_field(bytes = "0x20:0x21")]
    checksum: u16,

    /// The checksum of just the payload, before encryption
    #[packed_field(bytes = "0x34:0x35")]
    payload_checksum: u16,
}

impl CommandMessage {
    pub fn with_count<T>(count: u16, device_model_code: u16, mac: [u8; 6], id: u32) -> CommandMessage
    where
        T: CommandTrait
    {
        return CommandMessage{
            magic_header: [0x5A, 0xA5, 0xAA, 0x55, 0x5A, 0xA5, 0xaa, 0x55],
            device_type: device_model_code,
            packet_type: T::packet_type(),
            count: count | 0x8000,
            mac_reversed: reverse_mac(mac),
            id: id,
            checksum: 0,         // This will be populated later.
            payload_checksum: 0, // This will be populated later.
        };
    }

    pub fn new<T>(device_model_code: u16, mac: [u8; 6], id: u32) -> CommandMessage
    where
        T: CommandTrait
    {
        let mut r = rand::thread_rng();
        let random_count = r.gen_range(0x8000..0xFFFF);

        return CommandMessage::with_count::<T>(random_count, device_model_code, mac, id);
    }


    /// Pack the command message while appending the payload
    pub fn pack_with_payload(mut self, payload: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, String> {
        let cipher = AesCbc::new_from_slices(key, &constants::INITIAL_VECTOR)
            .expect("Could not construct cipher!");

        // Save the checksum of the payload before encrypting
        self.payload_checksum = checksum(&payload);

        // Encrypt the payload
        let encrypted = cipher.encrypt_vec(&payload);

        // Pack the command with the payload appended
        let packed = self.pack()
            .expect("Could not pack command header!");

        let mut appended = packed.to_vec();
        appended.extend(&encrypted);

        // Save the complete checksum
        self.checksum = checksum(&appended);

        // Construct the final message
        let completely_packed = self.pack()
            .expect("Could not pack completed command!");

        let mut complete_command: Vec<u8> = completely_packed.to_vec();
        complete_command.extend(&encrypted);

        return Ok(complete_command);
    }

    /// Unpack the command message with the associated payload
    pub fn unpack_with_payload(mut bytes: Vec<u8>, key: &[u8; 16]) -> Result<Vec<u8>, String> {
        // Ensure that the data is correct
        if bytes.len() < 0x38 {
            return Err(format!("Command is too short! Expected 0x38 bytes, got {}", bytes.len()));
        }

        // Unpack the header
        let command_header = CommandMessage::unpack_from_slice(&bytes[0..0x38])
            .expect("Could not unpack command from bytes!");

        // Zero out the checksum from the header for verification
        // TODO: Is there a nicer way to do this?
        bytes[0x20] = 0;
        bytes[0x21] = 0;

        // Ensure that the checksums match
        let real_checksum = checksum(&bytes);
        if command_header.checksum != real_checksum {
            return Err(format!(
                "Command checksum does not match actual checksum! Expected {} got {}",
                real_checksum,
                command_header.checksum,
            ));
        }

        // Decrypt the message
        let cipher = AesCbc::new_from_slices(key, &constants::INITIAL_VECTOR)
            .expect("Could not construct cipher!");

        let decrypted = cipher.decrypt_vec(&bytes[0x38..])
            .expect("Could not decrypt command payload!");

        return Ok(decrypted);
    }
}