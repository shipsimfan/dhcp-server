use self::leases::Leases;
use crate::{
    dhcp::{DHCPOptionClass, DHCPPacket, HardwareType, MessageType},
    IPAddress, MACAdddress,
};
use std::{collections::HashMap, net::SocketAddr};

mod leases;

pub struct DHCPServer {
    leases: Leases,
    reserved: HashMap<MACAdddress, IPAddress>,
}

#[derive(Debug)]
pub enum HandlePacketError {
    MalformedOption,
    NoMsgType,
    InvalidHardwareType(HardwareType),
    InvalidHardwareAddressLength(u8),
    NoIPAddressesAvailable,
}

impl DHCPServer {
    pub fn new() -> Self {
        let mut reserved = HashMap::new();
        for (mac, ip) in crate::config::RESERVED_IPS {
            reserved.insert(mac, ip);
        }

        DHCPServer {
            leases: Leases::new(),
            reserved,
        }
    }

    pub fn handle_packet(
        &mut self,
        packet: DHCPPacket,
    ) -> Result<Option<(DHCPPacket, Option<SocketAddr>)>, HandlePacketError> {
        // Ignore reply messages
        if packet.message_type() == MessageType::Reply {
            return Ok(None);
        }

        // Clear expired leases
        self.leases.clean_leases();

        // Get packet type
        let packet_type = match packet.get_option(DHCPOptionClass::DHCPMsgType) {
            Some(value) => match value.get(0) {
                Some(packet_type) => *packet_type,
                None => return Err(HandlePacketError::MalformedOption),
            },
            None => return Err(HandlePacketError::NoMsgType),
        };

        // Parse MACAddress
        let mac_address = match packet.hardware_type() {
            HardwareType::Ethernet => match packet.hardware_address_length() {
                6 => {
                    let address = packet.client_hardware_address();
                    MACAdddress::new([
                        address[0], address[1], address[2], address[3], address[4], address[5],
                    ])
                }
                _ => {
                    return Err(HandlePacketError::InvalidHardwareAddressLength(
                        packet.hardware_address_length(),
                    ))
                }
            },
            _ => {
                return Err(HandlePacketError::InvalidHardwareType(
                    packet.hardware_type(),
                ))
            }
        };

        // Parse packet type
        match packet_type {
            1 => {
                // Select an address for the new client
                let mut return_ip = None;

                if packet.client_ip_address() != IPAddress::new([0, 0, 0, 0]) {
                    // See if the client ip is available
                    // First in reserved
                    match self.reserved.get(&mac_address) {
                        Some(ip_address) => {
                            if *ip_address == packet.client_ip_address() {
                                return_ip = Some(*ip_address);
                            }
                        }
                        None => {
                            // Secondly in leases
                            match self.leases.get_ip_address(mac_address) {
                                Some(ip_address) => {
                                    if ip_address == packet.client_ip_address() {
                                        return_ip = Some(ip_address);
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                }

                let return_ip = match return_ip {
                    Some(return_ip) => return_ip,
                    None => {
                        // Check to see if there is a reserved address
                        let mut reserved_ip = None;
                        for (reserved_mac_address, ip_address) in &self.reserved {
                            if *reserved_mac_address == mac_address {
                                reserved_ip = Some(*ip_address);
                                break;
                            }
                        }

                        // Otherwise, allocate from lease
                        match reserved_ip {
                            Some(ip) => ip,
                            None => match self.leases.allocate() {
                                Some(ip) => ip,
                                None => return Err(HandlePacketError::NoIPAddressesAvailable),
                            },
                        }
                    }
                };

                // Send offer
                let mut packet = DHCPPacket::new(
                    packet.transaction_id(),
                    packet.flags(),
                    IPAddress::new([0, 0, 0, 0]),
                    return_ip,
                    crate::config::OUR_IP,
                    packet.gateway_ip_address(),
                    mac_address,
                );

                packet.add_option(DHCPOptionClass::DHCPMsgType, vec![53]);
                packet.add_option(
                    DHCPOptionClass::DHCPServerID,
                    vec![
                        crate::config::OUR_IP.get(0),
                        crate::config::OUR_IP.get(1),
                        crate::config::OUR_IP.get(2),
                        crate::config::OUR_IP.get(3),
                    ],
                );
                packet.add_option(DHCPOptionClass::AddressTime, vec![0x00, 0x02, 0xA3, 0x00]); // 2 Days
                packet.add_option(DHCPOptionClass::RenewalTime, vec![0x00, 0x01, 0x51, 0x80]); // 1 Day
                packet.add_option(DHCPOptionClass::RebindingTime, vec![0x00, 0x02, 0x4E, 0xA0]); // 1 Day, 18 Hours
                packet.add_option(DHCPOptionClass::SubnetMask, vec![255, 0, 0, 0]);
                packet.add_option(DHCPOptionClass::BroadcastAddress, vec![10, 255, 255, 255]);
                packet.add_option(
                    DHCPOptionClass::Gateways,
                    vec![
                        crate::config::GATEWAY_IP.get(0),
                        crate::config::GATEWAY_IP.get(1),
                        crate::config::GATEWAY_IP.get(2),
                        crate::config::GATEWAY_IP.get(3),
                    ],
                );
                packet.add_option(DHCPOptionClass::DomainServer, vec![1, 1, 1, 1, 1, 0, 0, 1]);
                packet.add_option(
                    DHCPOptionClass::ClientID,
                    vec![
                        HardwareType::Ethernet.generate(),
                        mac_address.get(0),
                        mac_address.get(1),
                        mac_address.get(2),
                        mac_address.get(3),
                        mac_address.get(4),
                        mac_address.get(5),
                    ],
                );
                packet.add_option(DHCPOptionClass::End, vec![]);

                return Ok(Some((packet, None)));
            }
            3 => println!("Request message recieved!"),
            4 => println!("Decline message recieved!"),
            8 => println!("Inform message recieved!"),
            _ => {} //Ok(None),
        }

        Ok(None)
    }
}

impl std::error::Error for HandlePacketError {}

impl std::fmt::Display for HandlePacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HandlePacketError::MalformedOption => format!("Malformed option"),
                HandlePacketError::NoMsgType => format!("No message type"),
                HandlePacketError::InvalidHardwareType(hardware_type) =>
                    format!("Invalid hardware type ({})", hardware_type),
                HandlePacketError::InvalidHardwareAddressLength(address_length) =>
                    format!("Invalid address length ({})", address_length),
                HandlePacketError::NoIPAddressesAvailable =>
                    format!("No I.P. Addresses are available"),
            }
        )
    }
}
