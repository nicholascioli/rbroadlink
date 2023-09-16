use packed_struct::prelude::{PackedStruct, PackedStructSlice, PrimitiveEnum_u8};

use crate::traits::CommandTrait;

/// The type of command to send with the remote's data.
#[derive(PrimitiveEnum_u8, Debug, Copy, Clone)]
pub enum RemoteDataCommand {
    /// Inform the device to send the attached IR/RF code
    SendCode = 0x02,

    /// Inform the device to start learning an IR code.
    StartLearningIR = 0x03,

    /// Inform the device to start learning an RF code.
    StartLearningRF = 0x1B,

    /// Inform the device to return the code learned.
    ///
    /// The device will send back no data if no code has been learned.
    GetCode = 0x04,

    /// Inform the device to start sweeping for RF frequencies.
    SweepRfFrequencies = 0x19,

    /// Inform the device to stop sweeping for RF frequencies.
    StopRfSweep = 0x1E,

    /// Inform the device to see if an RF frequency has been found during the sweep.
    CheckFrequency = 0x1A,
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
    /// Create a new RemoteDataMessage.
    pub fn new(command_type: RemoteDataCommand) -> RemoteDataMessage {
        return RemoteDataMessage {
            payload_length: 0,
            command: command_type,
        };
    }

    /// Pack the RemoteDataMessage with an associated payload.
    pub fn pack_with_payload(mut self, payload: &[u8]) -> Result<Vec<u8>, String> {
        // Calculate tyhe length of the payload
        self.payload_length = payload
            .len()
            .try_into()
            .map_err(|e| format!("Payload is too long! {}", e))?;

        // Add 4 for the needed stop sequence
        self.payload_length = self
            .payload_length
            .checked_add(4u16)
            .ok_or_else(|| "Could not add the start buffer! Payload is too long")?;

        // Append the payload to the header
        let mut result = self
            .pack()
            .map_err(|e| format!("Could not pack message! {}", e))?
            .to_vec();
        result.extend(payload);

        return Ok(result);
    }

    /// Unpack a RemoteDataMessage and return the associated payload.
    ///
    /// Note: The RemoteDataMessage will sometimes respond with unknown data,
    /// so this method returns no data at all if the response is not at least
    /// as large as the header.
    pub fn unpack_with_payload(bytes: &[u8]) -> Result<Vec<u8>, String> {
        // This is somewhat different than other messages. If there is no data, the
        // device will send us anywhere from 1 to 3 bytes, which is useless. So
        // we just discard anything that is below the threshold.
        if bytes.len() < 0x06 {
            return Ok(vec![]);
        }

        // Attempt to unpack the header
        let info = RemoteDataMessage::unpack_from_slice(&bytes[0x00..0x06])
            .map_err(|e| format!("Could not unpack remote data response! {}", e))?;

        // Extract the payload
        let payload_length = usize::from(info.payload_length + 1);
        let payload = &bytes[0x06..payload_length];

        return Ok(payload.to_vec());
    }
}

impl CommandTrait for RemoteDataMessage {
    fn packet_type() -> u16 {
        return 0x006A;
    }
}
