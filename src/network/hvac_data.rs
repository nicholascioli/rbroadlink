use packed_struct::prelude::{
    packed_bits, Integer, PackedStruct, PackedStructSlice, PrimitiveEnum_u8,
};

use crate::{network::util::compute_generic_checksum, traits::CommandTrait};

/// The type of command to send to the unit.
#[derive(PrimitiveEnum_u8, Debug, Copy, Clone)]
pub enum HvacDataCommand {
    /// Set New State
    SetState = 0x00,
    /// Get Current State
    GetState = 0x01,
    /// Obtain air conditioner basic information
    GetAcInfo = 0x02,
}

/// Enumerates modes.
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug)]
pub enum HvacMode {
    Auto = 0,
    Cool = 1,
    Dry = 2,
    Heat = 3,
    Fan = 4,
}

/// Enumerates fan speed.
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug)]
pub enum HvacSpeed {
    None = 0,
    High = 1,
    Mid = 2,
    Low = 3,
    Auto = 5,
}

/// Enumerates presets.
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug)]
pub enum HvacPreset {
    Normal = 0,
    Turbo = 1,
    Mute = 2,
}

/// Enumerates horizontal swing.
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug)]
pub enum HvacSwHoriz {
    LeftFix = 2,
    LeftRightFix = 7,
    RightFix = 6,
    RightFlap = 5,
    On = 0,
    Off = 1,
}

/// Enumerates vertical swing.
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug)]
pub enum HvacSwVert {
    On = 0,
    Pos1 = 1,
    Pos2 = 2,
    Pos3 = 3,
    Pos4 = 4,
    Pos5 = 5,
    Off = 7,
}

/// A struct with air conditioner state.
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "13")]
pub struct AirCondState {
    /// Power state (on/off)
    #[packed_field(bits = "66")]
    pub power: bool,

    /// Target temperature (integer)
    #[packed_field(bits = "0..=4")]
    target_temp_int: Integer<u8, packed_bits::Bits<5>>,
    // TODO the following fract field was not tested, commented out now
    // #[packed_field( bits="16")]
    // target_temp_fract: bool,
    /// Vertical swing
    #[packed_field(bits = "5..=7", ty = "enum")]
    pub swing_v: HvacSwVert,

    /// Horizontal swing
    #[packed_field(bits = "8..=10", ty = "enum")]
    pub swing_h: HvacSwHoriz,

    /// Device mode (heating, cooling, etc.)
    #[packed_field(bits = "40..=42", ty = "enum")]
    pub mode: HvacMode,

    /// Constant magic value
    #[packed_field(bits = "20..=23")]
    magic1: Integer<u8, packed_bits::Bits<4>>,

    /// Fan speed
    #[packed_field(bits = "24..=26", ty = "enum")]
    pub fanspeed: HvacSpeed,

    /// Preset (normal, turbo, etc.)
    #[packed_field(bits = "38..=39", ty = "enum")]
    pub preset: HvacPreset,

    /// Sleep mode
    #[packed_field(bits = "45")]
    pub sleep: bool,

    /// iFeel function (gets temperature from RCU)
    #[packed_field(bits = "44")]
    pub ifeel: bool,

    /// Health mode (clean the air by removing dust particles)
    #[packed_field(bits = "70")]
    pub health: bool,

    /// Auto-clean function
    /// prevent the growth of harmful microorganisms by eliminating the moisture inside of the indoor unit
    #[packed_field(bits = "69")]
    pub clean: bool,

    /// Enable/disable display showing current temperature
    #[packed_field(bits = "83")]
    pub display: bool,

    /// Dry mode (removes moisture as a major cause of mould and mildew in rooms)
    #[packed_field(bits = "84")]
    pub mildew: bool,
}

impl AirCondState {
    pub fn prepare_and_pack(&mut self) -> Result<Vec<u8>, String> {
        // set magic values before sending
        self.magic1 = 0x0f.into();

        Ok(self
            .pack()
            .map_err(|e| format!("Could not pack message! {}", e))?
            .to_vec())
    }

    /// Calculate final temperature value from internal partial fields.
    pub fn get_target_temp(&self) -> f32 {
        u8::from(self.target_temp_int) as f32 + 8.0
    }

    /// Set target temperature from input.
    pub fn set_target_temp(&mut self, input: f32) -> Result<(), String> {
        if input < 16.0 || input > 32.0 {
            return Err("Target temperature is out of range (16-32)".into());
        }
        // TODO: some units also have a 0.5 degree resolution, so in this
        // case the formula would be:
        // 8 + target_temp_int + target_temp_fract * 0.5
        // not tested, so currently only the integer:
        self.target_temp_int = (input as u8 - 8).into();

        Ok(())
    }
}

/// A struct with air conditioner basic info.
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "22")]
pub struct AirCondInfo {
    #[packed_field(bits = "15")]
    pub power: bool,
    #[packed_field(bits = "43..=47")]
    ambient_temp_int: Integer<u8, packed_bits::Bits<5>>,
    #[packed_field(bits = "171..=175")]
    ambient_temp_fract: Integer<u8, packed_bits::Bits<5>>,
}

impl AirCondInfo {
    /// Calculate final temperature value from internal partial fields.
    pub fn get_ambient_temp(&self) -> f32 {
        u8::from(self.ambient_temp_int) as f32 + u8::from(self.ambient_temp_fract) as f32 / 10.0
    }
}

/// A message used to communicate with the device.
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "12")]
pub struct HvacDataMessage {
    /// Length of the payload
    #[packed_field(bytes = "0x00:0x01")]
    payload_length: u16,

    #[packed_field(bytes = "0x02:0x03")]
    magic1: u16,
    #[packed_field(bytes = "0x04:0x05")]
    magic2: u16,
    #[packed_field(bytes = "0x06:0x07")]
    magic3: u16,

    #[packed_field(bytes = "0x08:0x09")]
    data_length: u16,

    /// Command flag for the message
    #[packed_field(bytes = "0x0a:0x0b")]
    command: u16,
}

impl HvacDataMessage {
    /// Create a new HvacDataMessage.
    pub fn new(command_type: HvacDataCommand) -> HvacDataMessage {
        return HvacDataMessage {
            payload_length: 0,
            command: (1 << 8) as u16 + ((command_type as u8) << 4 | 1) as u16,
            magic1: 0x00BBu16,
            magic2: 0x8006u16,
            magic3: 0,
            data_length: 2,
        };
    }

    /// Pack the HvacDataMessage with an associated payload.
    pub fn pack_with_payload(mut self, payload: &[u8]) -> Result<Vec<u8>, String> {
        // Calculate tyhe length of the payload
        self.data_length += <usize as TryInto<u16>>::try_into(payload.len())
            .map_err(|e| format!("Payload is too long! {}", e))?;

        // Add 10 bytes for the header
        self.payload_length = self
            .data_length
            .checked_add(10u16)
            .ok_or_else(|| "Could not add the start buffer! Payload is too long")?;

        // Append the payload to the header
        let mut result = self
            .pack()
            .map_err(|e| format!("Could not pack message! {}", e))?
            .to_vec();
        result.extend(payload);

        // Compute and add the final payload checksum
        let checksum = compute_generic_checksum(&result[2..]);
        result.extend(checksum.to_le_bytes().to_vec());

        return Ok(result);
    }

    /// Unpack a HvacDataMessage and return the associated payload.
    pub fn unpack_with_payload(bytes: &[u8]) -> Result<Vec<u8>, String> {
        // Unpack the header
        let command_header = HvacDataMessage::unpack_from_slice(&bytes[0..12])
            .map_err(|e| format!("Could not unpack command from bytes! {}", e))?;

        // Check total payload length:
        // get real size and substract 2 bytes length field for correct comparision
        let real_size: u16 = (bytes.len() as u16) - 2;
        if real_size != command_header.payload_length {
            return Err(format!(
                "Command checksum does not match actual checksum! Expected {:#06X} got {:#06X}",
                command_header.payload_length, real_size,
            ));
        }

        // Ensure that the checksums match
        let crc_offset = usize::from(command_header.payload_length);
        let data_crc = u16::from_le_bytes([bytes[crc_offset], bytes[crc_offset + 1]]);
        let real_checksum = compute_generic_checksum(&bytes[0x02..crc_offset]);
        if data_crc != real_checksum {
            return Err(format!(
                "Data checksum does not match actual checksum! Expected {:#06X} got {:#06X}",
                data_crc, real_checksum,
            ));
        }

        // Extract the data:
        // skip the first two bytes which probably contains the command code
        // returned by the device
        let data = &bytes[0x0C..0x0C + usize::from(command_header.data_length - 2)];

        return Ok(data.to_vec());
    }
}

impl CommandTrait for HvacDataMessage {
    fn packet_type() -> u16 {
        return 0x006A;
    }
}
