use std::{
    net::Ipv4Addr,
    time::Duration,
};

use phf::phf_map;

use crate::{
    DeviceInfo,
    Device,

    constants,
    network::{
        DiscoveryResponse,
        RemoteDataCommand,
        RemoteDataMessage,

        util::{
            reverse_mac,
        }
    },
};

const REMOTE_CODES: phf::Map<u16, &'static str> = phf_map! {
    0x6026u16 => "RM4 Pro",
    0x6184u16 => "RMC4 Pro",
    0x61A2u16 => "RM4 Pro",
    0x649Bu16 => "RM4 Pro",
    0x653Cu16 => "RM4 Pro",
};

#[derive(Debug, Clone)]
pub struct RemoteDevice {
    /// Base information about the remote.
    pub info: DeviceInfo,
}

impl RemoteDevice {
    pub fn new(name: &str, addr: Ipv4Addr, response: DiscoveryResponse) -> RemoteDevice {
        // Get the type of remote
        let friendly_model: String = REMOTE_CODES.get(&response.model_code)
            .unwrap_or(&"Unknown")
            .to_string();

        return Self{
            info: DeviceInfo{
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

    /// Attempts to learn an IR code.
    pub fn learn_ir(&self) -> Result<Vec<u8>, String> {
        // First enter learning...
        self.send_code(&[], RemoteDataCommand::StartLearning)
            .expect("Could not enter learning mode!");

        // Block until we learn the code or timeout
        let attempts = 10;
        let interval = Duration::from_secs(3);
        for _ in 0..attempts {
            // Sleep before trying again
            std::thread::sleep(interval);

            let code: Vec<u8> = self.send_code(&[], RemoteDataCommand::GetCode)
                .expect("Could not check code status of device!");
            if code.len() != 0 {
                return Ok(code);
            }
        }

        // If we haven't gotten anything up until now, then we failed
        return Err("Could not learn IR code! Operation timed out.".into());
    }

    /// Sends an IR code to the world.
    pub fn send_ir(&self, code: &[u8]) -> Result<(), String> {
        self.send_code(code, RemoteDataCommand::SendCode)
            .expect("Could not send IR code to device!");

        return Ok(());
    }

    /// Sends an RF code to the world.
    pub fn send_rf(&self, code: &[u8]) -> Result<(), String> {
        return Err("TODO".into());
    }

    /// Sends a raw code to the remote.
    /// Note: Try to avoid using this method in favor of send_ir, send_rf, etc.
    pub fn send_code(&self, payload: &[u8], command: RemoteDataCommand) -> Result<Vec<u8>, String> {
        // We cast this object to a generic device in order to make use of the shared
        // helper utilities.
        let generic_device = Device::Remote{ device: self.clone() };

        // Construct the data message
        let msg = RemoteDataMessage::new(command);
        let packed = msg.pack_with_payload(&payload)
            .expect("Could not pack remote data message!");

        let response = generic_device.send_command::<RemoteDataMessage>(&packed)
            .expect("Could not send code inside of the command!");

        return RemoteDataMessage::unpack_with_payload(&response);
    }
}
