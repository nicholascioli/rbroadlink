/// Represents a message that can be wrapped in a command.
pub trait CommandTrait {
    /// Returns the packet type expected by the broadlink device.
    fn packet_type() -> u16;
}
