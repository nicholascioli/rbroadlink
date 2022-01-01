use packed_struct::prelude::PrimitiveEnum_u16;

/// The different models of broadlink devices.
#[derive(PrimitiveEnum_u16, Clone, Copy, Debug)]
pub enum BroadlinkDevice {
    Rm4Pro = 0x649B,
    Unknown,
}
