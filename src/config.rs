use crate::{IPAddress, MACAdddress};

pub const LEASE_START_IP: IPAddress = IPAddress::new([10, 128, 0, 1]);
pub const GATEWAY_IP: IPAddress = IPAddress::new([10, 0, 0, 1]);
pub const OUR_IP: IPAddress = IPAddress::new([10, 0, 0, 2]);

pub const RESERVED_IPS: [(MACAdddress, IPAddress); 3] = [
    (
        MACAdddress::new([0x30, 0x9C, 0x23, 0x44, 0x17, 0x9B]),
        IPAddress::new([10, 1, 0, 1]),
    ),
    (
        MACAdddress::new([0xB6, 0xDC, 0x06, 0xA6, 0x6C, 0x19]),
        IPAddress::new([10, 1, 0, 2]),
    ),
    (
        MACAdddress::new([0xB8, 0x27, 0xEB, 0xBC, 0x3D, 0xF0]),
        IPAddress::new([10, 0, 0, 2]),
    ),
];
