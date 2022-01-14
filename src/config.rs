use crate::{IPAddress, MACAddress};
use std::env::args;

#[derive(Debug)]
pub struct Configuration {
    lease_start_ip: IPAddress,
    lease_final_ip: IPAddress,
    gateway_ip: IPAddress,
    our_ip: IPAddress,
    subnet_mask: IPAddress,
    broadcast_address: IPAddress,
    dns: IPAddress,
    dns_alternative: IPAddress,
    reserved_ips: Vec<(MACAddress, IPAddress)>,
    address_time: u32,
    renewal_time: u32,
    rebinding_time: u32,
    offer_time: u64,
}

#[derive(Debug)]
pub enum ConfigurationError {
    LoadError(config::Error),
    InvalidIP(String),
    InvalidMAC(String),
    InvalidTime(std::num::ParseIntError),
    NoLeaseStartIP,
    NoLeaseEndIP,
    NoGatewayIP,
    NoOurIP,
    NoSubnetMask,
    NoBroadcastAddress,
    NoDNS,
    NoAlternativeDNS,
    NoReservedIP(MACAddress),
}

const DEFAULT_ADDRESS_TIME: u32 = 60 * 60 * 48; // 2 Days
const DEFAULT_OFFER_TIME: u64 = 30; // 30 Seconds

const DEFAULT_CONFIGURATION_PATH: &'static str = "./config";

pub fn load_configuration() -> Result<Configuration, ConfigurationError> {
    let args: Vec<String> = args().collect();

    let configuration = config::Configuration::load(if args.len() > 1 {
        &args[1]
    } else {
        DEFAULT_CONFIGURATION_PATH
    })?;

    let lease_start_ip = match configuration.get("lease.start") {
        Some(str) => match IPAddress::parse(str) {
            Some(ip) => ip,
            None => return Err(ConfigurationError::InvalidIP(str.to_owned())),
        },
        None => return Err(ConfigurationError::NoLeaseStartIP),
    };

    let lease_final_ip = match configuration.get("lease.final") {
        Some(str) => match IPAddress::parse(str) {
            Some(ip) => ip,
            None => return Err(ConfigurationError::InvalidIP(str.to_owned())),
        },
        None => return Err(ConfigurationError::NoLeaseEndIP),
    };

    let gateway_ip = match configuration.get("gateway") {
        Some(str) => match IPAddress::parse(str) {
            Some(ip) => ip,
            None => return Err(ConfigurationError::InvalidIP(str.to_owned())),
        },
        None => return Err(ConfigurationError::NoGatewayIP),
    };

    let our_ip = match configuration.get("us") {
        Some(str) => match IPAddress::parse(str) {
            Some(ip) => ip,
            None => return Err(ConfigurationError::InvalidIP(str.to_owned())),
        },
        None => return Err(ConfigurationError::NoOurIP),
    };

    let subnet_mask = match configuration.get("subnet-mask") {
        Some(str) => match IPAddress::parse(str) {
            Some(ip) => ip,
            None => return Err(ConfigurationError::InvalidIP(str.to_owned())),
        },
        None => return Err(ConfigurationError::NoSubnetMask),
    };

    let broadcast_address = match configuration.get("broadcast") {
        Some(str) => match IPAddress::parse(str) {
            Some(ip) => ip,
            None => return Err(ConfigurationError::InvalidIP(str.to_owned())),
        },
        None => return Err(ConfigurationError::NoBroadcastAddress),
    };

    let dns = match configuration.get("dns.1") {
        Some(str) => match IPAddress::parse(str) {
            Some(ip) => ip,
            None => return Err(ConfigurationError::InvalidIP(str.to_owned())),
        },
        None => return Err(ConfigurationError::NoDNS),
    };

    let dns_alternative = match configuration.get("dns.2") {
        Some(str) => match IPAddress::parse(str) {
            Some(ip) => ip,
            None => return Err(ConfigurationError::InvalidIP(str.to_owned())),
        },
        None => return Err(ConfigurationError::NoAlternativeDNS),
    };

    let mut i = 0;
    let mut reserved_ips = Vec::new();
    loop {
        let mac_address = match configuration.get(&format!("reserved.{}.mac", i)) {
            Some(str) => match MACAddress::parse(str) {
                Some(mac_address) => mac_address,
                None => return Err(ConfigurationError::InvalidMAC(str.to_owned())),
            },
            None => break,
        };

        let ip_address = match configuration.get(&format!("reserved.{}.ip", i)) {
            Some(str) => match IPAddress::parse(str) {
                Some(ip) => ip,
                None => return Err(ConfigurationError::InvalidIP(str.to_owned())),
            },
            None => return Err(ConfigurationError::NoReservedIP(mac_address)),
        };

        reserved_ips.push((mac_address, ip_address));
        i += 1;
    }

    let address_time = match configuration.get("lease.time") {
        Some(str) => match str.parse() {
            Ok(value) => value,
            Err(error) => return Err(ConfigurationError::InvalidTime(error)),
        },
        None => DEFAULT_ADDRESS_TIME,
    };

    let renewal_time = match configuration.get("renewal-time") {
        Some(str) => match str.parse() {
            Ok(value) => value,
            Err(error) => return Err(ConfigurationError::InvalidTime(error)),
        },
        None => address_time / 2,
    };

    let rebinding_time = match configuration.get("rebinding-time") {
        Some(str) => match str.parse() {
            Ok(value) => value,
            Err(error) => return Err(ConfigurationError::InvalidTime(error)),
        },
        None => (renewal_time / 2) * 3,
    };

    let offer_time = match configuration.get("offer-time") {
        Some(str) => match str.parse() {
            Ok(value) => value,
            Err(error) => return Err(ConfigurationError::InvalidTime(error)),
        },
        None => DEFAULT_OFFER_TIME,
    };

    Ok(Configuration {
        lease_start_ip,
        lease_final_ip,
        gateway_ip,
        our_ip,
        subnet_mask,
        broadcast_address,
        dns,
        dns_alternative,
        reserved_ips,
        address_time,
        renewal_time,
        rebinding_time,
        offer_time,
    })
}

impl Configuration {
    pub fn reserved_ips(&self) -> &Vec<(MACAddress, IPAddress)> {
        &self.reserved_ips
    }

    pub fn our_ip(&self) -> IPAddress {
        self.our_ip
    }

    pub fn gateway_ip(&self) -> IPAddress {
        self.gateway_ip
    }

    pub fn subnet_mask(&self) -> IPAddress {
        self.subnet_mask
    }

    pub fn broadcast_address(&self) -> IPAddress {
        self.broadcast_address
    }

    pub fn dns(&self) -> (IPAddress, IPAddress) {
        (self.dns, self.dns_alternative)
    }

    pub fn lease_start_ip(&self) -> IPAddress {
        self.lease_start_ip
    }

    pub fn lease_final_ip(&self) -> IPAddress {
        self.lease_final_ip
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

    pub fn offer_time(&self) -> u64 {
        self.offer_time
    }
}

impl std::fmt::Display for Configuration {
    /*
            lease_start_ip,
        lease_final_ip,
        gateway_ip,
        our_ip,
        subnet_mask,
        broadcast_address,
        dns,
        dns_alternative,
        reserved_ips,
        address_time,
        renewal_time,
        rebinding_time,
        offer_time,
    */
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  Lease:")?;
        writeln!(f, "    Start: {}", self.lease_start_ip)?;
        writeln!(f, "    End: {}", self.lease_final_ip)?;
        writeln!(f, "    Time: {}", self.address_time)?;
        writeln!(f, "  Our I.P.: {}", self.our_ip)?;
        writeln!(f, "  Gateway I.P.: {}", self.gateway_ip)?;
        writeln!(f, "  Subnet Mask: {}", self.subnet_mask)?;
        writeln!(f, "  Broadcast Address: {}", self.broadcast_address)?;
        writeln!(f, "  DNS: ({}, {})", self.dns, self.dns_alternative)?;

        writeln!(f, "  Reservations:")?;
        for (mac, ip) in &self.reserved_ips {
            writeln!(f, "    {} -> {}", mac, ip)?;
        }

        writeln!(f, "Renewal Time: {}", self.renewal_time)?;
        writeln!(f, "Rebinding Time: {}", self.rebinding_time)?;
        writeln!(f, "Offer Time: {}", self.offer_time)
    }
}

impl std::error::Error for ConfigurationError {}

impl std::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ConfigurationError::LoadError(error) =>
                    format!("Unable to load configuration file - {}", error),
                ConfigurationError::InvalidIP(str) => format!("Invalid I.P. address ({})", str),
                ConfigurationError::InvalidMAC(str) => format!("Invalid MAC address ({})", str),
                ConfigurationError::InvalidTime(str) => format!("Invalid time ({})", str),
                ConfigurationError::NoLeaseStartIP => format!("No lease start I.P. address"),
                ConfigurationError::NoLeaseEndIP => format!("No lease end I.P. address"),
                ConfigurationError::NoGatewayIP => format!("No gateway I.P. address"),
                ConfigurationError::NoOurIP => format!("Our I.P. address not specified"),
                ConfigurationError::NoSubnetMask => format!("No subnet mask"),
                ConfigurationError::NoBroadcastAddress => format!("No broadcast address"),
                ConfigurationError::NoDNS => format!("No DNS specified"),
                ConfigurationError::NoAlternativeDNS => format!("No alternative DNS specified"),
                ConfigurationError::NoReservedIP(address) => format!(
                    "Reserved MAC Address has no match I.P. address ({})",
                    address
                ),
            }
        )
    }
}

impl From<config::Error> for ConfigurationError {
    fn from(error: config::Error) -> Self {
        ConfigurationError::LoadError(error)
    }
}
