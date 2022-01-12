use crate::{IPAddress, MACAdddress};
use std::{collections::HashMap, time::Instant};

pub struct Leases {
    leases: HashMap<IPAddress, (MACAdddress, Instant)>,
    offers: HashMap<IPAddress, Instant>,
    next_available_ip: Option<IPAddress>,
}

impl Leases {
    pub fn new() -> Self {
        Leases {
            leases: HashMap::new(),
            offers: HashMap::new(),
            next_available_ip: Some(crate::config::LEASE_START_IP),
        }
    }

    pub fn clean_leases(&mut self) {
        // TODO: implement clearing expired leases & offers
    }

    pub fn allocate(&mut self) -> Option<IPAddress> {
        let ret = match self.next_available_ip {
            Some(ip) => ip,
            None => return None,
        };

        // Search for next available I.P.
        let mut ip = ret;
        self.next_available_ip = None;
        while ip <= crate::config::LEASE_FINAL_IP {
            match self.leases.get(&ip) {
                Some(_) => {}
                None => match self.offers.get(&ip) {
                    Some(_) => {}
                    None => {
                        self.next_available_ip = Some(ip);
                        break;
                    }
                },
            }

            ip.increament();
        }

        // Reserve the offer
        self.offers.insert(ret, Instant::now());

        Some(ret)
    }

    pub fn get_ip_address(&self, mac_address: MACAdddress) -> Option<IPAddress> {
        let mut ip_address = crate::config::LEASE_START_IP;
        while ip_address <= crate::config::LEASE_FINAL_IP {
            match self.leases.get(&ip_address) {
                Some((lease_address, _)) => {
                    if *lease_address == mac_address {
                        return Some(ip_address);
                    }
                }
                None => {}
            }

            ip_address.increament();
        }

        None
    }

    pub fn get_mac_address(&self, ip_address: IPAddress) -> Option<MACAdddress> {
        self.leases
            .get(&ip_address)
            .map(|(mac_address, _)| *mac_address)
    }
}
