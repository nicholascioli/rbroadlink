use packed_struct::prelude::PackedStruct;

use crate::network::util::checksum;

/// WirelessConnection represents the credentials for connecting to a wireless
/// network.
#[derive(Debug)]
pub enum WirelessConnection<'a> {
    /// None represents a network with no security
    None(&'a str),

    /// WEP represents a network with WEP security key.
    WEP(&'a str, &'a str),

    /// WPA1 represents a network with a WPA v1 security key.
    WPA1(&'a str, &'a str),

    /// WPA represents a network with a WPA v2 security key.
    WPA2(&'a str, &'a str),

    /// WPA represents a network with both WPA v1 and WPA v2 support.
    WPA(&'a str, &'a str),
}

/// WirelessConnectionMessage represents a message used for instructing a device
/// to connect to a specified wireless network.
///
/// Refer to the following for struct layout -> https://github.com/mjg59/python-broadlink/blob/9ff6b2d48e58f005765088cdf3dc5cc553cdb01a/protocol.md
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "136")]
pub struct WirelessConnectionMessage {
    /// The message's checksum for verification purposes.
    #[packed_field(bytes = "32:33")]
    checksum: u16,

    /// A magic constant for this message. Always 0x14
    #[packed_field(bytes = "38")]
    magic_constant: i8,

    /// The SSID of the network.
    #[packed_field(bytes = "68:99")]
    ssid: [u8; 32],

    /// The password of the network, if needed.
    #[packed_field(bytes = "100:131")]
    password: [u8; 32],

    /// Length of the SSID
    #[packed_field(bytes = "132")]
    ssid_length: u8,

    /// Length of the password
    #[packed_field(bytes = "133")]
    password_length: u8,

    /// The security mode to use. 0 -> None, 1 -> WEP, 2 -> WPA1, 3 -> WPA2, 4 -> WPA
    #[packed_field(bytes = "134")]
    security_mode: u8,
}

impl WirelessConnection<'_> {
    pub fn to_message(&self) -> Result<WirelessConnectionMessage, String> {
        let empty_pass = "";
        let (ssid, pass, security_mode) = match self {
            WirelessConnection::None(ssid) => (ssid, &empty_pass, 0),
            WirelessConnection::WEP(ssid, pass) => (ssid, pass, 1),
            WirelessConnection::WPA1(ssid, pass) => (ssid, pass, 2),
            WirelessConnection::WPA2(ssid, pass) => (ssid, pass, 3),
            WirelessConnection::WPA(ssid, pass) => (ssid, pass, 4),
        };

        // Ensure that the fields aren't too long
        if ssid.len() > 32 {
            return Err("Could not use provided SSID! SSID longer than 32 characters.".into());
        }

        // Copy over the strings into their fixed buffers
        let mut ssid_fixed = [0u8; 32];
        let mut pass_fixed = [0u8; 32];
        for (i, c) in ssid.as_bytes().iter().enumerate() {
            ssid_fixed[i] = *c;
        }
        for (i, c) in pass.as_bytes().iter().enumerate() {
            pass_fixed[i] = *c;
        }

        // Construct the message
        let mut msg = WirelessConnectionMessage{
            // We will need to recalculate this after creating the message
            checksum: 0,

            // This is always 0x14
            magic_constant: 0x14,

            // Grab info from connection
            ssid: ssid_fixed,
            password: pass_fixed,
            ssid_length: u8::try_from(ssid.len()).expect("Could not use provided SSID! SSID is too long (max 32 characters)."),
            password_length: u8::try_from(pass.len()).expect("Could not use provided password! Password is too long (max 32 characters)."),

            security_mode: security_mode,
        };

        // Add the checksum into the msg
        msg.checksum = checksum(
            &msg.pack().expect("Could not pack WirelessConnectionMessage!")
        );

        // Return the newly created message
        return Ok(msg);
    }
}