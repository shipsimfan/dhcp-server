mod hardware_type;
mod message_type;
mod option;
mod packet;

pub use hardware_type::*;
pub use message_type::*;
pub use option::*;
pub use packet::*;

fn slice_to_u16(slice: &[u8]) -> u16 {
    (slice[0] as u16) << 8 | slice[1] as u16
}

fn slice_to_u32(slice: &[u8]) -> u32 {
    (slice[0] as u32) << 24 | (slice[1] as u32) << 16 | (slice[2] as u32) << 8 | slice[3] as u32
}

fn u16_to_slice(value: u16) -> [u8; 2] {
    [(value.wrapping_shr(8) & 0xFF) as u8, (value & 0xFF) as u8]
}

fn u32_to_slice(value: u32) -> [u8; 4] {
    [
        (value.wrapping_shr(24) & 0xFF) as u8,
        (value.wrapping_shr(16) & 0xFF) as u8,
        (value.wrapping_shr(8) & 0xFF) as u8,
        (value & 0xFF) as u8,
    ]
}
