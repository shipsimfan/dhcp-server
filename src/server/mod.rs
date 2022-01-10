use self::leases::Leases;
use crate::{
    dhcp::{DHCPOptionClass, DHCPPacket, MessageType},
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

        // Get packet type
        let packet_type = match packet.get_option(DHCPOptionClass::DHCPMsgType) {
            Some(value) => match value.get(0) {
                Some(packet_type) => *packet_type,
                None => return Err(HandlePacketError::MalformedOption),
            },
            None => return Err(HandlePacketError::NoMsgType),
        };

        // Parse packet type
        match packet_type {
            1 => println!("Discover message recieved!"),
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
                HandlePacketError::MalformedOption => "Malformed option",
                HandlePacketError::NoMsgType => "No message type",
            }
        )
    }
}
