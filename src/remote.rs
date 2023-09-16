use std::{net::Ipv4Addr, time::Duration};

use phf::phf_map;

use crate::{
    constants,
    network::{util::reverse_mac, DiscoveryResponse, RemoteDataCommand, RemoteDataMessage},
    Device, DeviceInfo,
};

/// A mapping of remote device codes to their friendly model equivalent.
pub const REMOTE_CODES: phf::Map<u16, &'static str> = phf_map! {
    0x520Bu16 => "RM4 Pro",
    0x5213u16 => "RM4 Pro",
    0x5218u16 => "RM4C Pro",
    0x6026u16 => "RM4 Pro",
    0x6184u16 => "RMC4 Pro",
    0x61A2u16 => "RM4 Pro",
    0x649Bu16 => "RM4 Pro",
    0x653Cu16 => "RM4 Pro",
};

/// A broadlink device capable of transmitting IR / RF codes.
#[derive(Debug, Clone)]
pub struct RemoteDevice {
    /// Base information about the remote.
    pub info: DeviceInfo,
}

impl RemoteDevice {
    /// Create a new RemoteDevice.
    ///
    /// Note: This should not be called directly. Please use [Device::from_ip] or
    /// [Device::list] instead.
    pub fn new(name: &str, addr: Ipv4Addr, response: DiscoveryResponse) -> RemoteDevice {
        // Get the type of remote
        let friendly_model: String = REMOTE_CODES
            .get(&response.model_code)
            .unwrap_or(&"Unknown")
            .to_string();

        return Self {
            info: DeviceInfo {
                address: addr,
                mac: reverse_mac(response.mac),
                model_code: response.model_code,
                friendly_type: "Remote".into(),
                friendly_model: friendly_model,
                name: name.into(),
                auth_id: 0, // This will be populated when authenticated.
                key: constants::INITIAL_KEY,
                is_locked: response.is_locked,
            },
        };
    }

    /// Attempt to learn an IR code.
    ///
    /// When learning, the remote's LED will light up orange. Simply long press
    /// (and release) the IR button while pointing the control at the device until the light
    /// turns off.
    pub fn learn_ir(&self) -> Result<Vec<u8>, String> {
        // First enter learning...
        self.send_command(&[], RemoteDataCommand::StartLearningIR)
            .map_err(|e| format!("Could not enter learning mode! {}", e))?;

        // Block until we learn the code or timeout
        let attempts = 10;
        let interval = Duration::from_secs(3);
        for _ in 0..attempts {
            // Sleep before trying again
            std::thread::sleep(interval);

            let code: Vec<u8> = self
                .send_command(&[], RemoteDataCommand::GetCode)
                .map_err(|e| format!("Could not check code status of device! {}", e))?;
            if code.len() != 0 {
                return Ok(code);
            }
        }

        // If we haven't gotten anything up until now, then we failed
        return Err("Could not learn IR code! Operation timed out.".into());
    }

    /// Attempts to learn an RF code.
    ///
    /// The device must go through two stages in order to learn an RF code.
    ///   1) It must learn which RF frequency is in use.
    ///   2) It must learn the actual RF code.
    ///
    /// To do so, follow these instructions:
    ///   1) Wait for the device's LED to turn orange
    ///   2) Long press (and release) the RF button until the orange LED turns off
    ///      and then back on.
    ///   3) Press the RF button once more normally until the orange LED turns off.
    pub fn learn_rf(&self) -> Result<Vec<u8>, String> {
        // Start sweeping for the type of frequency in use
        self.send_command(&[], RemoteDataCommand::SweepRfFrequencies)
            .map_err(|e| format!("Could not start sweeping frequencies! {}", e))?;

        // Wait for the frequency to be identified
        let attempts = 10;
        let interval = Duration::from_secs(3);
        let mut frequency_found = false;
        for _ in 0..attempts {
            // Sleep before trying again
            std::thread::sleep(interval);

            let frequency: Vec<u8> = self
                .send_command(&[], RemoteDataCommand::CheckFrequency)
                .map_err(|e| format!("Could not check code status of device! {}", e))?;
            if frequency[0] == 1 {
                frequency_found = true;
                break;
            }
        }

        // Error out if no frequency is found
        if !frequency_found {
            self.send_command(&[], RemoteDataCommand::StopRfSweep)
                .map_err(|e| format!("Could not cancel RF sweep! {}", e))?;
            return Err("Could not determine frequency!".into());
        }

        // Enter RF learning mode
        self.send_command(&[], RemoteDataCommand::StartLearningRF)
            .map_err(|e| format!("Could not enter learning mode! {}", e))?;

        // Block until we learn the code or timeout
        for _ in 0..attempts {
            // Sleep before trying again
            std::thread::sleep(interval);

            let code: Vec<u8> = self
                .send_command(&[], RemoteDataCommand::GetCode)
                .map_err(|e| format!("Could not check code status of device! {}", e))?;
            if code.len() != 0 {
                return Ok(code);
            }
        }

        // If we haven't gotten anything up until now, then we failed
        self.send_command(&[], RemoteDataCommand::StopRfSweep)
            .map_err(|e| format!("Could not cancel RF sweep! {}", e))?;
        return Err("Could not learn RF code! Operation timed out.".into());
    }

    /// Sends an IR/RF code to the world.
    pub fn send_code(&self, code: &[u8]) -> Result<(), String> {
        self.send_command(code, RemoteDataCommand::SendCode)
            .map_err(|e| format!("Could not send IR code to device! {}", e))?;

        return Ok(());
    }

    /// Sends a raw command to the remote.
    /// Note: Try to avoid using this method in favor of [RemoteDevice::send_code], [RemoteDevice::learn_ir], etc.
    pub fn send_command(
        &self,
        payload: &[u8],
        command: RemoteDataCommand,
    ) -> Result<Vec<u8>, String> {
        // We cast this object to a generic device in order to make use of the shared
        // helper utilities.
        let generic_device = Device::Remote {
            remote: self.clone(),
        };

        // Construct the data message
        let msg = RemoteDataMessage::new(command);
        let packed = msg
            .pack_with_payload(&payload)
            .map_err(|e| format!("Could not pack remote data message! {}", e))?;

        let response = generic_device
            .send_command::<RemoteDataMessage>(&packed)
            .map_err(|e| format!("Could not send code inside of the command! {}", e))?;

        return RemoteDataMessage::unpack_with_payload(&response);
    }
}
