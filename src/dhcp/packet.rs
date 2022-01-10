use super::{DHCPOption, DHCPOptionClass, HardwareType, MessageType};
use crate::IPAddress;

pub struct DHCPPacket {
    message_type: MessageType,
    hardware_type: HardwareType,
    hardware_address_length: u8,
    hops: u8,
    transaction_id: u32,
    seconds: u16,
    flags: u16,
    client_ip_address: IPAddress,
    your_ip_address: IPAddress,
    server_ip_address: IPAddress,
    gateway_ip_address: IPAddress,
    client_hardware_address: [u8; 16],
    options: Vec<DHCPOption>,
}

#[derive(Debug)]
pub enum PacketParseError {
    TooShort(usize),
    NoEndOption,
    InvalidMagic([u8; 4]),
    InvalidMessageType(u8),
    ParseOptionError(super::ParseOptionError),
}

impl DHCPPacket {
    pub fn parse(packet: &[u8]) -> Result<Self, PacketParseError> {
        if packet.len() < 241 {
            return Err(PacketParseError::TooShort(packet.len()));
        }

        // Parse basic information
        let message_type = match MessageType::parse(packet[0]) {
            Some(message_type) => message_type,
            None => return Err(PacketParseError::InvalidMessageType(packet[0])),
        };
        let hardware_type = HardwareType::parse(packet[1]);
        let hardware_address_length = packet[2];
        let hops = packet[3];
        let transaction_id = super::slice_to_u32(&packet[4..]);
        let seconds = super::slice_to_u16(&packet[8..]);
        let flags = super::slice_to_u16(&packet[10..]);
        let client_ip_address = IPAddress::new([packet[12], packet[13], packet[14], packet[15]]);
        let your_ip_address = IPAddress::new([packet[16], packet[17], packet[18], packet[19]]);
        let server_ip_address = IPAddress::new([packet[20], packet[21], packet[22], packet[23]]);
        let gateway_ip_address = IPAddress::new([packet[24], packet[25], packet[26], packet[27]]);
        let client_hardware_address = [
            packet[28], packet[29], packet[30], packet[31], packet[32], packet[33], packet[34],
            packet[35], packet[36], packet[37], packet[38], packet[39], packet[40], packet[41],
            packet[42], packet[43],
        ];

        // Parse options
        let mut options = Vec::new();
        let mut i = 240;
        loop {
            if i >= packet.len() {
                return Err(PacketParseError::NoEndOption);
            }

            let option = DHCPOption::parse(&packet[i..])?;

            if option.class() == DHCPOptionClass::End {
                break;
            }

            i += option.value().len() + 2;

            options.push(option);
        }

        if packet[236] != 99 || packet[237] != 130 || packet[238] != 83 || packet[239] != 99 {
            Err(PacketParseError::InvalidMagic([
                packet[236],
                packet[237],
                packet[238],
                packet[239],
            ]))
        } else {
            Ok(DHCPPacket {
                message_type,
                hardware_type,
                hardware_address_length,
                hops,
                transaction_id,
                seconds,
                flags,
                client_ip_address,
                your_ip_address,
                server_ip_address,
                gateway_ip_address,
                client_hardware_address,
                options,
            })
        }
    }
}

impl std::fmt::Display for DHCPPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message Type: {}\n", self.message_type)?;
        write!(f, "Hardware Type: {}\n", self.hardware_type)?;
        write!(
            f,
            "Hardware Address Length: {}\n",
            self.hardware_address_length
        )?;
        write!(f, "Hops: {}\n", self.hops)?;
        write!(f, "Transaction ID: {}\n", self.transaction_id)?;
        write!(f, "Seconds: {}\n", self.seconds)?;
        write!(f, "Flags: {}\n", self.flags)?;
        write!(f, "Client I.P. Address: {}\n", self.client_ip_address)?;
        write!(f, "Your I.P. Address: {}\n", self.your_ip_address)?;
        write!(f, "Server I.P. Address: {}\n", self.server_ip_address)?;
        write!(f, "Gateway I.P. Address: {}\n", self.gateway_ip_address)?;
        write!(
            f,
            "Client Hardware Address: {:?}\n",
            self.client_hardware_address
        )?;
        write!(f, "Options:\n")?;
        for option in &self.options {
            write!(f, "    {}\n", option)?;
        }
        Ok(())
    }
}

impl std::error::Error for PacketParseError {}

impl std::fmt::Display for PacketParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PacketParseError::TooShort(length) => format!("Too short ({})", length),
                PacketParseError::NoEndOption => format!("No end option"),
                PacketParseError::InvalidMagic(magic) =>
                    format!("Invalid magic value ({:?})", magic),
                PacketParseError::InvalidMessageType(message_type) =>
                    format!("Invalid message type ({})", message_type),
                PacketParseError::ParseOptionError(error) =>
                    format!("Unable to parse option ({})", error),
            }
        )
    }
}

impl From<super::ParseOptionError> for PacketParseError {
    fn from(error: super::ParseOptionError) -> Self {
        PacketParseError::ParseOptionError(error)
    }
}
