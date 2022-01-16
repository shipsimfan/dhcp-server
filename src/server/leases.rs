use crate::{IPAddress, MACAddress};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub struct Leases {
    leases: HashMap<IPAddress, (MACAddress, Instant)>,
    offers: HashMap<IPAddress, (MACAddress, Instant)>,
    next_available_ip: Option<IPAddress>,
    start: IPAddress,
    end: IPAddress,
    offer_time: u64,
    address_time: u32,
    renewal_time: u32,
    rebinding_time: u32,
}

impl Leases {
    pub fn new(configuration: &crate::config::Configuration) -> Self {
        Leases {
            leases: HashMap::new(),
            offers: HashMap::new(),
            next_available_ip: Some(configuration.lease_start_ip()),
            start: configuration.lease_start_ip(),
            end: configuration.lease_final_ip(),
            address_time: configuration.address_time(),
            renewal_time: configuration.renewal_time(),
            rebinding_time: configuration.rebinding_time(),
            offer_time: configuration.offer_time(),
        }
    }

    pub fn clean_leases(&mut self) {
        let logger = logging::get_logger(module_path!());

        self.offers.retain(|ip, (mac, offer_time)| {
            if offer_time.elapsed() < Duration::from_secs(self.offer_time) {
                true
            } else {
                logging::info!(logger, "Removed lease for {} to {} due to expiry", ip, mac);
                false
            }
        });

        self.leases.retain(|ip, (mac, lease_time)| {
            if lease_time.elapsed() < Duration::from_secs(self.address_time as u64) {
                true
            } else {
                logging::info!(logger, "Removed offer for {} to {} due to expiry", ip, mac);
                false
            }
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
        while ip <= self.end {
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
        if ip_address < self.start || ip_address > self.end {
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

    pub fn release(&mut self, ip_address: IPAddress, mac_address: MACAddress) {
        self.leases
            .retain(|ip, (mac, _)| *ip != ip_address || *mac != mac_address)
    }

    pub fn address_time(&self) -> u32 {
        self.address_time
    }

    pub fn renewal_time(&self) -> u32 {
        self.renewal_time
    }

    pub fn rebinding_time(&self) -> u32 {
        self.rebinding_time
    }

    pub fn current_leases(&self) -> Vec<(IPAddress, MACAddress)> {
        let mut ret = Vec::new();

        for (ip, (mac, _)) in &self.leases {
            ret.push((*ip, *mac));
        }

        ret
    }
}
