use std::{collections::HashMap, time::Instant};

#[derive(Clone, Copy)]
pub struct IPAddress([u8; 4]);

#[derive(Clone, Copy)]
pub struct MACAdddress([u8; 6]);

struct LeasedAddress(IPAddress, MACAdddress);

struct Leases {
    leases: [Option<(MACAdddress, Instant)>; 254],
    next_available_ip: IPAddress,
}

pub struct DHCPServer {
    leases: Leases,
    reserved: HashMap<MACAdddress, IPAddress>,
}

impl DHCPServer {
    pub fn new() -> Self {
        DHCPServer {
            leases: Leases::new(),
            reserved: HashMap::new(),
        }
    }
}

impl Leases {
    pub fn new() -> Self {
        Leases {
            leases: [None; 254],
            next_available_ip: crate::config::LEASE_START_IP,
        }
    }
}

impl IPAddress {
    pub const fn new(address: [u8; 4]) -> Self {
        IPAddress(address)
    }
}

impl MACAdddress {
    pub const fn new(address: [u8; 6]) -> Self {
        MACAdddress(address)
    }
}
