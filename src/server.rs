#![allow(unused)]

use crate::{IPAddress, MACAdddress};
use std::{collections::HashMap, time::Instant};

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
