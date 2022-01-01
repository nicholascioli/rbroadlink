use packed_struct::prelude::PackedStruct;

use crate::network::CommandWrapped;

/// A message used to send data to be blasted to a broadlink devices on the network.
#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb", size_bytes = "48")]
pub struct DataMessage {
    //
}

impl CommandWrapped for DataMessage {
    fn packet_type() -> u16 {
        return 0x006A;
    }
}
