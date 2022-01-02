//! Set of utility methods useful when working with network requests.

use std::{
    net::{
        IpAddr,
        Ipv4Addr,
        SocketAddr,
        UdpSocket,
    },
    time::Duration,
};

/// Computes the checksum of a slice of bytes.
///
/// The checksum is computed by summing all of the bytes with 0xBEAF and masking
/// with 0xFFFF.
pub fn checksum(data: &[u8]) -> u16 {
    // Get the checksum
    let mut sum = 0xBEAFu32;
    for &d in data {
        sum += u32::from(d);
    }

    return sum as u16;
}

/// Returns the first available non-local address or the passed IP, if present.
pub fn local_ip_or(ip: Option<Ipv4Addr>) -> IpAddr {
    return match ip {
        Some(ip) => IpAddr::V4(ip),
        None => get_if_addrs::get_if_addrs()
            .expect("Could not automatically determine machine IP address")
            .iter()
            .find(|x| x.ip().is_ipv4() && !x.ip().is_loopback())
            .expect("Could not find a local IPv4 address!")
            .ip(),
    };
}

/// Sends a message and returns the received response.
fn send_and_receive_impl(msg: &[u8], addr: Ipv4Addr, port: Option<u16>) -> Result<UdpSocket, String> {
    // Set up the socket addresses
    let unspecified_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port.unwrap_or(0)));
    let destination_addr = SocketAddr::from((addr, 80));

    // Set up the communication socket
    // Note: We need to enable support for broadcast
    let socket = UdpSocket::bind(unspecified_addr)
        .expect("Could not bind to any port.");
    socket.set_broadcast(true)
        .expect("Could not enable broadcast.");

    // Send the message
    socket.set_read_timeout(Some(Duration::new(10, 0))).expect("Could not set read timeout!");
    socket.send_to(&msg, destination_addr).expect("Could not broadcast message!");

    return Ok(socket);
}

/// Sends a message and returns the as many received responses as possible (within a timeout).
pub fn send_and_receive_many<I, T>(msg: &[u8], addr: Ipv4Addr, port: Option<u16>, cb: T) -> Result<Vec<I>, String>
where
    T: Fn(usize, &[u8], SocketAddr) -> Result<I, String>
{
    // Get the socket
    let socket = send_and_receive_impl(msg, addr, port)
        .expect("Could not create socket for message sending!");

    // Transform the results
    let mut results: Vec<I> = vec![];
    let mut recv_buffer = [0u8; 8092];
    while let Ok((bytes_received, addr)) = socket.recv_from(&mut recv_buffer) {
        results.push(
            cb(bytes_received, &recv_buffer[0..bytes_received], addr).expect("Could not map result!")
        );
    }

    return Ok(results);
}

/// Sends a message and returns the first received response.
pub fn send_and_receive_one<I, T>(msg: &[u8], addr: Ipv4Addr, port: Option<u16>, cb: T) -> Result<I, String>
where
    T: Fn(usize, &[u8], SocketAddr) -> Result<I, String>
{
    // Get the socket
    let socket = send_and_receive_impl(msg, addr, port)
        .expect("Could not create socket for message sending!");

    // Transform the result
    let mut recv_buffer = [0u8; 8092];
    if let Ok((bytes_received, addr)) = socket.recv_from(&mut recv_buffer) {
        return Ok(
            cb(bytes_received, &recv_buffer[0..bytes_received], addr).expect("Could not map result!")
        );
    }

    return Err("No response within timeout!".into());
}

/// Reverses a MAC address. Used to fix the backwards response from the broadlink device.
pub fn reverse_mac(mac_flipped: [u8; 6]) -> [u8; 6] {
    // Fix the mac address by reversing it.
    let mut mac = [0u8; 6];
    for i in 0..6 {
        mac[i] = mac_flipped[6 - i - 1];
    }

    return mac;
}