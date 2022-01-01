use packed_struct::prelude::{ PackedStruct, PackedStructSlice, PrimitiveEnum_u8 };

use crate::traits::CommandTrait;

#[derive(PrimitiveEnum_u8, Debug, Copy, Clone)]
pub enum RemoteDataCommand {
    SendCode = 0x02,
    StartLearning = 0x03,
    GetCode = 0x04,
}

/// A message used to inform a remote of data to blast.
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "0x06")]
pub struct RemoteDataMessage {
    /// Length of the payload
    #[packed_field(bytes = "0x00:0x01")]
    payload_length: u16,

    /// Command flag for the message
    #[packed_field(bytes = "0x02", ty = "enum")]
    command: RemoteDataCommand,
}

impl RemoteDataMessage {
    pub fn new(command_type: RemoteDataCommand) -> RemoteDataMessage {
        return RemoteDataMessage {
            payload_length: 0,
            command: command_type,
        };
    }

    pub fn pack_with_payload(mut self, payload: &[u8]) -> Result<Vec<u8>, String> {
        // Calculate tyhe length of the payload
        self.payload_length = payload
            .len()
            .try_into()
            .expect("Payload is too long!");

        // Add 4 for the needed stop sequence
        self.payload_length = self.payload_length
            .checked_add(4u16)
            .expect("Could not add the start buffer! Payload is too long");

        // Append the payload to the header
        let mut result = self.pack()
            .expect("Could not pack message!")
            .to_vec();
        result.extend(payload);

        return Ok(result);
    }

    pub fn unpack_with_payload(bytes: &[u8]) -> Result<Vec<u8>, String> {
        // This is somewhat different than other messages. If there is no data, the
        // device will send us anywhere from 1 to 3 bytes, which is useless. So
        // we just discard anything that is below the threshold.
        if bytes.len() < 0x06 {
            return Ok(vec![]);
        }

        // Attempt to unpack the header
        let info = RemoteDataMessage::unpack_from_slice(&bytes[0x00..0x02])
            .expect("Could not unpack remote data response!");

        // Extract the payload
        let payload_length = usize::from(info.payload_length + 2);
        let payload = &bytes[0x06..payload_length];

        return Ok(payload.to_vec());
    }
}

impl CommandTrait for RemoteDataMessage {
    fn packet_type() -> u16 {
        return 0x006A;
    }
}