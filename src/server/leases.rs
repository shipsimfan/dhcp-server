use crate::{IPAddress, MACAdddress};
use std::time::Instant;

struct LeasedAddress(IPAddress, MACAdddress);

pub struct Leases {
    leases: [Option<(MACAdddress, Instant)>; 254],
    next_available_ip: IPAddress,
}

impl Leases {
    pub fn new() -> Self {
        Leases {
            leases: [None; 254],
            next_available_ip: crate::config::LEASE_START_IP,
        }
    }
}
