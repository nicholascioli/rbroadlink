use crate::DeviceInfo;

pub trait DeviceTrait {
    /// Get the core information about a device.
    fn get_info(&self) -> DeviceInfo;

    /// Save the authentication information
    fn save_auth_pair(&mut self, id: u32, key: [u8; 16]);
}