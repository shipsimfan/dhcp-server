use crate::{IPAddress, MACAddress};

pub const LEASE_START_IP: IPAddress = IPAddress::new([10, 128, 0, 1]);
pub const LEASE_FINAL_IP: IPAddress = IPAddress::new([10, 255, 255, 254]);
pub const GATEWAY_IP: IPAddress = IPAddress::new([10, 0, 0, 1]);
pub const OUR_IP: IPAddress = IPAddress::new([10, 0, 0, 2]);
pub const SUBNET_MASK: IPAddress = IPAddress::new([255, 0, 0, 0]);
pub const BROADCAST_ADDRESS: IPAddress = IPAddress::new([10, 255, 255, 255]);
pub const DNS: IPAddress = IPAddress::new([1, 1, 1, 1]);
pub const DNS_ALTERNATIVE: IPAddress = IPAddress::new([1, 0, 0, 1]);

pub const RESERVED_IPS: [(MACAddress, IPAddress); 3] = [
    (
        MACAddress::new([0x30, 0x9C, 0x23, 0x44, 0x17, 0x9B]),
        IPAddress::new([10, 1, 0, 1]),
    ),
    (
        MACAddress::new([0xB6, 0xDC, 0x06, 0xA6, 0x6C, 0x19]),
        IPAddress::new([10, 1, 0, 2]),
    ),
    (
        MACAddress::new([0xB8, 0x27, 0xEB, 0xBC, 0x3D, 0xF0]),
        IPAddress::new([10, 0, 0, 2]),
    ),
];

pub const ADDRESS_TIME: u32 = 60 * 60 * 48; // 2 Days
pub const RENEWAL_TIME: u32 = 60 * 60 * 24; // 1 Day
pub const REBINDING_TIME: u32 = 60 * 60 * 36; // 1 Day 18 Hours
pub const OFFER_TIME: u64 = 30; // 30 Seconds
