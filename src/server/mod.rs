use self::leases::Leases;
use crate::{
    dhcp::{DHCPOptionClass, DHCPPacket, HardwareType, MessageType},
    u32_to_slice, IPAddress, MACAddress,
};
use std::{collections::HashMap, net::SocketAddr};

mod leases;

pub struct DHCPServer {
    leases: Leases,
    reserved: HashMap<MACAddress, IPAddress>,
}

#[derive(Debug)]
pub enum HandlePacketError {
    MalformedOption,
    NoMsgType,
    InvalidHardwareType(HardwareType),
    InvalidHardwareAddressLength(u8),
    NoIPAddressesAvailable,
    InvalidRequestedAddressLength,
    NoRequestedIPInRequest,
    InvalidRenewAddress,
    DeclineMessageRecieved,
}

pub const DHCP_SERVER_PORT: u16 = 67;
pub const DHCP_CLIENT_PORT: u16 = 68;

const DHCP_MESSAGE_TYPE_DISCOVER: u8 = 1;
const DHCP_MESSAGE_TYPE_OFFER: u8 = 2;
const DHCP_MESSAGE_TYPE_REQUEST: u8 = 3;
const DHCP_MESSAGE_TYPE_DECLINE: u8 = 4;
const DHCP_MESSAGE_TYPE_ACK: u8 = 5;
const DHCP_MESSAGE_TYPE_NACK: u8 = 6;
const DHCP_MESSAGE_TYPE_RELEASE: u8 = 7;
const DHCP_MESSAGE_TYPE_INFORM: u8 = 8;

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
                    MACAddress::new([
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
            DHCP_MESSAGE_TYPE_DISCOVER => {
                self.handle_discover_packet(&packet, mac_address)
                    .map(move |response_packet| {
                        Some((
                            response_packet,
                            if packet.gateway_ip_address() != IPAddress::new([0, 0, 0, 0]) {
                                Some(packet.gateway_ip_address().to_socket_addr(DHCP_SERVER_PORT))
                            } else {
                                if packet.client_ip_address() != IPAddress::new([0, 0, 0, 0]) {
                                    Some(
                                        packet.client_ip_address().to_socket_addr(DHCP_CLIENT_PORT),
                                    )
                                } else {
                                    None
                                }
                            },
                        ))
                    })
            }
            DHCP_MESSAGE_TYPE_REQUEST => self
                .handle_request_packet(packet, mac_address)
                .map(|response| Some(response)),
            DHCP_MESSAGE_TYPE_DECLINE => Err(HandlePacketError::DeclineMessageRecieved),
            DHCP_MESSAGE_TYPE_RELEASE => {
                self.leases.release(packet.client_ip_address(), mac_address);
                Ok(None)
            }
            DHCP_MESSAGE_TYPE_INFORM => {
                Ok(Some(self.generate_ack_packet(packet, None, mac_address)))
            }
            _ => Ok(None),
        }
    }

    fn handle_discover_packet(
        &mut self,
        packet: &DHCPPacket,
        mac_address: MACAddress,
    ) -> Result<DHCPPacket, HandlePacketError> {
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
                    None => match self.leases.allocate(mac_address) {
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
            *packet.client_hardware_address(),
        );

        packet.add_option(DHCPOptionClass::DHCPMsgType, &[DHCP_MESSAGE_TYPE_OFFER]);
        packet.add_option(
            DHCPOptionClass::DHCPServerID,
            crate::config::OUR_IP.as_slice(),
        );
        packet.add_option(
            DHCPOptionClass::AddressTime,
            &u32_to_slice(crate::config::ADDRESS_TIME),
        );
        packet.add_option(
            DHCPOptionClass::RenewalTime,
            &u32_to_slice(crate::config::RENEWAL_TIME),
        );
        packet.add_option(
            DHCPOptionClass::RebindingTime,
            &u32_to_slice(crate::config::REBINDING_TIME),
        ); // 1 Day, 18 Hours
        packet.add_option(
            DHCPOptionClass::SubnetMask,
            crate::config::SUBNET_MASK.as_slice(),
        );
        packet.add_option(
            DHCPOptionClass::BroadcastAddress,
            crate::config::BROADCAST_ADDRESS.as_slice(),
        );
        packet.add_option(
            DHCPOptionClass::Gateways,
            crate::config::GATEWAY_IP.as_slice(),
        );
        let mut dns = Vec::from(crate::config::DNS.as_slice());
        dns.extend_from_slice(crate::config::DNS_ALTERNATIVE.as_slice());
        packet.add_option(DHCPOptionClass::DomainServer, dns.as_slice());
        let mut client_id = vec![HardwareType::Ethernet.generate()];
        client_id.extend_from_slice(mac_address.as_slice());
        packet.add_option(DHCPOptionClass::ClientID, &client_id.as_slice());
        packet.add_option(DHCPOptionClass::End, &[]);

        Ok(packet)
    }

    fn handle_request_packet(
        &mut self,
        packet: DHCPPacket,
        mac_address: MACAddress,
    ) -> Result<(DHCPPacket, Option<SocketAddr>), HandlePacketError> {
        // Get requested I.P. address
        let requested_ip = match packet.get_option(DHCPOptionClass::AddressRequest) {
            Some(value) => {
                if value.len() != 4 {
                    return Err(HandlePacketError::InvalidRequestedAddressLength);
                } else {
                    IPAddress::new([value[0], value[1], value[2], value[3]])
                }
            }
            None => {
                if packet.client_ip_address() == IPAddress::new([0, 0, 0, 0]) {
                    return Err(HandlePacketError::NoRequestedIPInRequest);
                } else {
                    // Renewing / Rebinding

                    // Verify client i.p. before responding
                    match self.reserved.get(&mac_address) {
                        Some(ip) => {
                            if *ip == packet.client_ip_address() {
                                return Ok(self.generate_ack_packet(
                                    packet,
                                    Some(*ip),
                                    mac_address,
                                ));
                            } else {
                                return Err(HandlePacketError::InvalidRenewAddress);
                            }
                        }
                        None => {
                            if self
                                .leases
                                .accept_offer(packet.client_ip_address(), mac_address)
                            {
                                let requested_ip = packet.client_ip_address();
                                return Ok(self.generate_ack_packet(
                                    packet,
                                    Some(requested_ip),
                                    mac_address,
                                ));
                            } else {
                                return Err(HandlePacketError::InvalidRenewAddress);
                            }
                        }
                    }
                }
            }
        };

        // See if client has reserved I.P. Address
        match self.reserved.get(&mac_address) {
            Some(ip_address) => {
                // Has a reserved I.P. address
                if requested_ip == *ip_address {
                    // Requesting reserved I.P. address
                    return Ok(self.generate_ack_packet(packet, Some(requested_ip), mac_address));
                } else {
                    // Requesting another I.P. address than one that is reserved
                    return Ok((self.generate_nack_packet(packet, mac_address), None));
                }
            }
            None => {} // No reserved I.P. address
        }

        // Verify requested I.P. with leases
        if self.leases.accept_offer(requested_ip, mac_address) {
            Ok(self.generate_ack_packet(packet, Some(requested_ip), mac_address))
        } else {
            Ok((self.generate_nack_packet(packet, mac_address), None))
        }
    }

    fn generate_ack_packet(
        &self,
        request_packet: DHCPPacket,
        requested_address: Option<IPAddress>,
        mac_address: MACAddress,
    ) -> (DHCPPacket, Option<SocketAddr>) {
        let mut packet = DHCPPacket::new(
            request_packet.transaction_id(),
            request_packet.flags(),
            request_packet.client_ip_address(),
            match requested_address {
                Some(address) => address,
                None => IPAddress::new([0, 0, 0, 0]),
            },
            crate::config::OUR_IP,
            request_packet.gateway_ip_address(),
            *request_packet.client_hardware_address(),
        );

        packet.add_option(DHCPOptionClass::DHCPMsgType, &[DHCP_MESSAGE_TYPE_ACK]);
        packet.add_option(
            DHCPOptionClass::DHCPServerID,
            crate::config::OUR_IP.as_slice(),
        );

        if requested_address.is_some() {
            packet.add_option(
                DHCPOptionClass::AddressTime,
                &u32_to_slice(crate::config::ADDRESS_TIME),
            );
            packet.add_option(
                DHCPOptionClass::RenewalTime,
                &u32_to_slice(crate::config::RENEWAL_TIME),
            );
            packet.add_option(
                DHCPOptionClass::RebindingTime,
                &u32_to_slice(crate::config::REBINDING_TIME),
            );
        }

        packet.add_option(
            DHCPOptionClass::SubnetMask,
            crate::config::SUBNET_MASK.as_slice(),
        );
        packet.add_option(
            DHCPOptionClass::BroadcastAddress,
            crate::config::BROADCAST_ADDRESS.as_slice(),
        );
        packet.add_option(
            DHCPOptionClass::Gateways,
            crate::config::GATEWAY_IP.as_slice(),
        );
        let mut dns = Vec::from(crate::config::DNS.as_slice());
        dns.extend_from_slice(crate::config::DNS_ALTERNATIVE.as_slice());
        packet.add_option(DHCPOptionClass::DomainServer, dns.as_slice());
        let mut client_id = vec![HardwareType::Ethernet.generate()];
        client_id.extend_from_slice(mac_address.as_slice());
        packet.add_option(DHCPOptionClass::ClientID, &client_id.as_slice());
        packet.add_option(DHCPOptionClass::End, &[]);

        (
            packet,
            match requested_address {
                Some(_) => {
                    if request_packet.gateway_ip_address() != IPAddress::new([0, 0, 0, 0]) {
                        Some(
                            request_packet
                                .gateway_ip_address()
                                .to_socket_addr(DHCP_SERVER_PORT),
                        )
                    } else {
                        if request_packet.client_ip_address() != IPAddress::new([0, 0, 0, 0]) {
                            Some(
                                request_packet
                                    .client_ip_address()
                                    .to_socket_addr(DHCP_CLIENT_PORT),
                            )
                        } else {
                            None
                        }
                    }
                }
                None => Some(
                    request_packet
                        .client_ip_address()
                        .to_socket_addr(DHCP_CLIENT_PORT),
                ),
            },
        )
    }

    fn generate_nack_packet(
        &self,
        request_packet: DHCPPacket,
        mac_address: MACAddress,
    ) -> DHCPPacket {
        let mut packet = DHCPPacket::new(
            request_packet.transaction_id(),
            request_packet.flags(),
            IPAddress::new([0, 0, 0, 0]),
            IPAddress::new([0, 0, 0, 0]),
            IPAddress::new([0, 0, 0, 0]),
            request_packet.gateway_ip_address(),
            *request_packet.client_hardware_address(),
        );

        packet.add_option(DHCPOptionClass::DHCPMsgType, &[DHCP_MESSAGE_TYPE_NACK]);
        packet.add_option(
            DHCPOptionClass::DHCPServerID,
            crate::config::OUR_IP.as_slice(),
        );
        packet.add_option(DHCPOptionClass::ClientID, mac_address.as_slice());
        packet.add_option(DHCPOptionClass::End, &[]);

        packet
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
                HandlePacketError::InvalidRequestedAddressLength =>
                    format!("Invalid requested address length"),
                HandlePacketError::NoRequestedIPInRequest =>
                    format!("No requested address in request"),
                HandlePacketError::InvalidRenewAddress => format!("Invalid renew address"),
                HandlePacketError::DeclineMessageRecieved => format!("Decline packet recieved"),
            }
        )
    }
}
