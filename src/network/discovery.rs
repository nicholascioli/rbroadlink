use std::net::IpAddr;

use chrono::prelude::{ Datelike, DateTime, Local, Timelike };
use packed_struct::prelude::PackedStruct;

use crate::{
    network::util::checksum,
};

/// A message used to discover all broadlink devices on the network.
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "48")]
pub struct DiscoveryMessage {
    /// Current offset from GMT
    #[packed_field(bytes = "8:11")]
    gmt_offset: i32,

    /// Current year of this request.
    #[packed_field(bytes = "12:13")]
    year: u16,

    /// Current number of minutes past the hour of this request.
    #[packed_field(bytes = "14")]
    minute: u8,

    /// Current number of hours past midnight of this request.
    #[packed_field(bytes = "15")]
    hour: u8,

    /// Current year, without the century (ex. 00, 01, ...)
    #[packed_field(bytes = "16")]
    year_without_century: u8,

    /// Current day of the week of this request. Monday is 1, Tuesday is 0, etc...
    #[packed_field(bytes = "17")]
    day_of_the_week: u8,

    /// Current day of the month of this request.
    #[packed_field(bytes = "18")]
    day_of_the_month: u8,

    /// Current month of this request.
    #[packed_field(bytes = "19")]
    month: u8,

    /// Listening IP of the requesting machine, reversed. Note: Only IPv4 is allowed here.
    #[packed_field(bytes = "24:27")]
    local_ip_reversed: [u8; 4],

    /// Listening port of the requesting machine.
    #[packed_field(bytes = "28:29")]
    local_port: u16,

    /// The message's checksum for verification purposes.
    #[packed_field(bytes = "32:33")]
    checksum: u16,

    /// Magic code for this message. Always 0x06
    #[packed_field(bytes = "38")]
    magic_constant: u8,
}

/// A valid response from a discovery message.
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "128")]
pub struct DiscoveryResponse {
    /// Device model. Refer to the BroadlinkDevice enum for more info.
    #[packed_field(bytes = "52:53")]
    pub model_code: u16,

    /// Device MAC address.
    #[packed_field(bytes = "58:63")]
    pub mac: [u8; 6],

    /// Device Name
    #[packed_field(bytes = "64:126")]
    pub name: [u8; 62],

    /// Device lock status
    #[packed_field(bytes = "127")]
    pub is_locked: bool,
}

impl DiscoveryMessage {
    /// Create a new DiscoveryMessage.
    pub fn new(addr: IpAddr, port: u16, time: Option<DateTime<Local>>) -> Result<DiscoveryMessage, String> {
        // Get the time
        let time = match time {
            Some(t) => t,
            None => Local::now(),
        };

        // Get the ip addr. Note: The device only supports IPv4
        let selected_ip = match addr {
            IpAddr::V4(ipv4) => ipv4,
            _ => return Err("Could not construct DiscoveryMessage! IP address is not IPv4".into())
        };

        // Reverse the IP octet
        // Note: This is needed since the packet expects it to be reversed due to LSB
        let octets = selected_ip.octets();
        let reversed_ip: [u8; 4] = [octets[3], octets[2], octets[1], octets[0]];

        // Chrono returns the information in u32, so we need to convert them here.
        // These conversions should, in theory, not fail. But we check nonetheless.
        let mut msg = DiscoveryMessage {
            gmt_offset: time.offset().local_minus_utc(),
            year: time.year().try_into().expect("Could not construct DiscoveryMessage! Year is out of range."),
            minute: time.minute().try_into().expect("Could not construct DiscoveryMessage! Minutes are out of range."),
            hour: time.hour().try_into().expect("Could not construct DiscoveryMessage! Hour is out of range."),
            year_without_century: (time.year() % 100) as u8,
            day_of_the_week: time.weekday().number_from_monday().try_into().expect("Could not construct DiscoveryMessage! Day opf the week is out of range."),
            day_of_the_month: time.day().try_into().expect("Could not construct DiscoveryMessage! Day is out of range."),
            month: time.month().try_into().expect("Could not construct DiscoveryMessage! Month is out of range."),
            local_ip_reversed: reversed_ip,
            local_port: port,

            // This will be filled in later
            checksum: 0,
            magic_constant: 0x06,
        };

        // Calculate the checksum
        msg.checksum = checksum(
            &msg.pack().expect("Could not pack DiscoveryMessage!")
        );

        return Ok(msg);
    }
}
