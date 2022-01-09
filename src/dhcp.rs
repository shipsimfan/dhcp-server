use crate::server::IPAddress;

struct DHCPOption {
    class: u8,
    length: u8,
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
    pub fn parse(packet: &[u8]) -> Result<Self, ()> {
        if packet.len() < 241 {
            return Err(());
        }

        let message_type = packet[0];
        let hardware_type = packet[1];
        let hardware_address_length = packet[2];
        let hops = packet[3];
        let transaction_id = slice_to_u32(&packet[4..])?;
        let seconds = slice_to_u16(&packet[8..])?;
        let flags = slice_to_u16(&packet[10..])?;
        let client_ip_address = IPAddress::new([packet[12], packet[13], packet[14], packet[15]]);
        let your_ip_address = IPAddress::new([packet[16], packet[17], packet[18], packet[19]]);
        let server_ip_address = IPAddress::new([packet[20], packet[21], packet[22], packet[23]]);
        let gateway_ip_address = IPAddress::new([packet[24], packet[25], packet[26], packet[27]]);
        let client_hardware_address = [
            packet[28], packet[29], packet[30], packet[31], packet[32], packet[33], packet[34],
            packet[35], packet[36], packet[37], packet[38], packet[39], packet[40], packet[41],
            packet[42], packet[43],
        ];

        let mut options = Vec::new();
        let mut i = 240;
        loop {
            if i >= packet.len() {
                return Err(());
            }

            let class = packet[i];
            if class == 255 {
                break;
            }

            i += 1;
            if i >= packet.len() {
                return Err(());
            }

            let length = packet[i];

            let mut value = Vec::new();
            for _ in 0..length {
                i += 1;
                if i >= packet.len() {
                    return Err(());
                }

                value.push(packet[i]);
            }

            options.push(DHCPOption::new(class, length, value))
        }

        if packet[236] != b'D' || packet[237] != b'H' || packet[238] != b'C' || packet[239] != b'P'
        {
            Err(())
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
    pub fn new(class: u8, length: u8, value: Vec<u8>) -> Self {
        DHCPOption {
            class,
            length,
            value,
        }
    }
}
