use std::net::Ipv4Addr;

use packed_struct::PackedStructSlice;
use phf::phf_map;

use crate::{
    constants,
    network::{
        util::reverse_mac, AirCondInfo, AirCondState, DiscoveryResponse, HvacDataCommand,
        HvacDataMessage,
    },
    Device, DeviceInfo,
};

/// A mapping of hvac device codes to their friendly model equivalent.
pub const HVAC_CODES: phf::Map<u16, &'static str> = phf_map! {
    0x4E2Au16 => "Licensed manufacturer",
};

/// A broadlink HVAC/Air Conditioner device.
#[derive(Debug, Clone)]
pub struct HvacDevice {
    /// Base information about the device.
    pub info: DeviceInfo,
}

impl HvacDevice {
    /// Create a new HvacDevice.
    ///
    /// Note: This should not be called directly. Please use [Device::from_ip] or
    /// [Device::list] instead.
    pub fn new(name: &str, addr: Ipv4Addr, response: DiscoveryResponse) -> HvacDevice {
        // Get the name of air conditioner
        let friendly_model: String = HVAC_CODES
            .get(&response.model_code)
            .unwrap_or(&"Unknown")
            .to_string();

        return Self {
            info: DeviceInfo {
                address: addr,
                mac: reverse_mac(response.mac),
                model_code: response.model_code,
                friendly_type: "HVAC".into(),
                friendly_model: friendly_model,
                name: name.into(),
                auth_id: 0, // This will be populated when authenticated.
                key: constants::INITIAL_KEY,
                is_locked: response.is_locked,
            },
        };
    }

    /// Get basic information from the air conditioner.
    pub fn get_info(&self) -> Result<AirCondInfo, String> {
        let data = self
            .send_command(&[], HvacDataCommand::GetAcInfo)
            .map_err(|e| format!("Could not obtain AC info from device! {}", e))?;
        let info = AirCondInfo::unpack_from_slice(&data)
            .map_err(|e| format!("Could not unpack command from bytes! {}", e))?;

        return Ok(info);
    }

    /// Get current air conditioner state into AirCondState structure.
    pub fn get_state(&self) -> Result<AirCondState, String> {
        let data = self
            .send_command(&[], HvacDataCommand::GetState)
            .map_err(|e| format!("Could not obtain AC state from device! {}", e))?;
        let state = AirCondState::unpack_from_slice(&data)
            .map_err(|e| format!("Could not unpack command from bytes! {}", e))?;

        return Ok(state);
    }

    /// Set new air conditioner state based on passed structure.
    pub fn set_state(&self, state: &mut AirCondState) -> Result<Vec<u8>, String> {
        let payload = state
            .prepare_and_pack()
            .map_err(|e| format!("Could not pack message! {}", e))?;
        let response = self.send_command(&payload, HvacDataCommand::SetState)?;

        return Ok(response);
    }

    /// Sends a raw command to the device.
    /// Note: Try to avoid using this method in favor of [HvacDevice::get_info], [HvacDevice::set_state], etc.
    pub fn send_command(
        &self,
        payload: &[u8],
        command: HvacDataCommand,
    ) -> Result<Vec<u8>, String> {
        // We cast this object to a generic device in order to make use of the shared
        // helper utilities.
        let generic_device = Device::Hvac { hvac: self.clone() };

        // Construct the data message
        let msg = HvacDataMessage::new(command);
        let packed = msg
            .pack_with_payload(&payload)
            .map_err(|e| format!("Could not pack HVAC data message! {}", e))?;

        let response = generic_device
            .send_command::<HvacDataMessage>(&packed)
            .map_err(|e| format!("Could not send command! {}", e))?;

        // TODO: check if there is some relation between
        // msg.command and the same return field from the response

        return HvacDataMessage::unpack_with_payload(&response);
    }
}
