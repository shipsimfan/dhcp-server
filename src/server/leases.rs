use crate::{IPAddress, MACAddress};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub struct Leases {
    leases: HashMap<IPAddress, (MACAddress, Instant)>,
    offers: HashMap<IPAddress, (MACAddress, Instant)>,
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
        self.offers.retain(|_, (_, offer_time)| {
            offer_time.elapsed() < Duration::from_secs(crate::config::OFFER_TIME)
        });

        self.leases.retain(|_, (_, lease_time)| {
            lease_time.elapsed() < Duration::from_secs(crate::config::ADDRESS_TIME as u64)
        });
    }

    pub fn allocate(&mut self, mac_address: MACAddress) -> Option<IPAddress> {
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
        self.offers.insert(ret, (mac_address, Instant::now()));

        Some(ret)
    }

    pub fn accept_offer(&mut self, ip_address: IPAddress, mac_address: MACAddress) -> bool {
        // Verify I.P. range
        if ip_address < crate::config::LEASE_START_IP || ip_address > crate::config::LEASE_FINAL_IP
        {
            return false;
        }

        // Check offers
        match self.offers.get(&ip_address) {
            Some((mac, _)) => {
                if mac_address != *mac {
                    return false;
                }
            }
            None => {
                // Check leases
                match self.leases.get(&ip_address) {
                    Some((mac, _)) => {
                        if mac_address != *mac {
                            return false;
                        }
                    }
                    None => {}
                }
            }
        }

        self.offers.remove(&ip_address);
        self.leases
            .insert(ip_address, (mac_address, Instant::now()));
        true
    }

    pub fn get_ip_address(&self, mac_address: MACAddress) -> Option<IPAddress> {
        for (ip, (mac, _)) in &self.leases {
            if *mac == mac_address {
                return Some(*ip);
            }
        }

        None
    }
}
