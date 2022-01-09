use crate::IPAddress;

struct DHCPOption {
    class: u8,
    value: Vec<u8>,
}

pub struct DHCPPacket {
    message_type: u8,
    hardware_type: u8,
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
    TooShort,
    NoEndOption,
    InvalidOptionLength,
    InvalidMagic,
}

fn slice_to_u16(slice: &[u8]) -> Result<u16, ()> {
    if slice.len() < 2 {
        Err(())
    } else {
        Ok((slice[0] as u16) << 8 | slice[1] as u16)
    }
}

fn slice_to_u32(slice: &[u8]) -> Result<u32, ()> {
    if slice.len() < 4 {
        Err(())
    } else {
        Ok((slice[0] as u32) << 24
            | (slice[1] as u32) << 16
            | (slice[2] as u32) << 8
            | slice[3] as u32)
    }
}

impl DHCPPacket {
    pub fn parse(packet: &[u8]) -> Result<Self, PacketParseError> {
        if packet.len() < 241 {
            return Err(PacketParseError::TooShort);
        }

        // Parse basic information
        let message_type = packet[0];
        let hardware_type = packet[1];
        let hardware_address_length = packet[2];
        let hops = packet[3];
        let transaction_id = slice_to_u32(&packet[4..]).unwrap();
        let seconds = slice_to_u16(&packet[8..]).unwrap();
        let flags = slice_to_u16(&packet[10..]).unwrap();
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

            let class = packet[i];
            if class == 255 {
                break;
            }

            i += 1;
            if i >= packet.len() {
                return Err(PacketParseError::NoEndOption);
            }

            let length = packet[i];

            let mut value = Vec::new();
            for _ in 0..length {
                i += 1;
                if i >= packet.len() {
                    return Err(PacketParseError::InvalidOptionLength);
                }

                value.push(packet[i]);
            }

            options.push(DHCPOption::new(class, value))
        }

        if packet[236] != 99 || packet[237] != 130 || packet[238] != 83 || packet[239] != 99 {
            Err(PacketParseError::InvalidMagic)
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

impl DHCPOption {
    pub fn new(class: u8, value: Vec<u8>) -> Self {
        DHCPOption { class, value }
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

impl std::fmt::Display for DHCPOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {:?}", self.class, self.value)
    }
}

impl std::error::Error for PacketParseError {}

impl std::fmt::Display for PacketParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PacketParseError::TooShort => "Too short",
                PacketParseError::NoEndOption => "No end option",
                PacketParseError::InvalidOptionLength => "Invalid option length",
                PacketParseError::InvalidMagic => "Invalid magic value",
            }
        )
    }
}
