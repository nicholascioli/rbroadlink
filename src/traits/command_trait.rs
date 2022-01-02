/// Represents a message that can be wrapped in a command.
pub trait CommandTrait {
    /// Returns the packet type expected by a [crate::network::CommandMessage] sent to a broadlink device.
    fn packet_type() -> u16;
}
