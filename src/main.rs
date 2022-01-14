use std::{net::UdpSocket, process::exit};

mod address;
mod config;
mod dhcp;
mod server;
mod util;

pub use address::*;
pub use util::*;

#[derive(Debug)]
enum RuntimeError {
    CreateServerError(std::io::Error),
    LoadConfigurationError(config::ConfigurationError),
}

#[derive(Debug)]
enum RequestError {
    ReadRequestError(std::io::Error),
    ParsePacketError(dhcp::PacketParseError),
    HandlePacketError(server::HandlePacketError),
    WriteResponseError(std::io::Error),
}

const BROADCAST_ADDRESS: IPAddress = IPAddress::new([255, 255, 255, 255]);

fn print_fatal_error(error: RuntimeError) -> ! {
    eprintln!("\x1B[31;1mFatal Error:\x1B[0m {}", error);
    exit(1);
}

fn print_request_error(error: RequestError) {
    eprintln!("\x1B[31;1mError with client:\x1B[0m {}", error);
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => print_fatal_error(error),
    }
}

fn run() -> Result<(), RuntimeError> {
    // Load configuration
    let configuration = config::load_configuration()?;

    println!("{}", configuration);

    // Create DHCP Server
    let mut server = server::DHCPServer::new(configuration);

    // Create UDP Server
    let mut socket = match UdpSocket::bind(format!("0.0.0.0:{}", server::DHCP_SERVER_PORT)) {
        Ok(socket) => socket,
        Err(error) => return Err(RuntimeError::CreateServerError(error)),
    };

    match socket.set_broadcast(true) {
        Ok(()) => {}
        Err(error) => return Err(RuntimeError::CreateServerError(error)),
    };

    println!("DHCP Server listening on port {}", server::DHCP_SERVER_PORT);

    // Handle requests
    loop {
        match handle_request(&mut socket, &mut server) {
            Ok(()) => {}
            Err(error) => print_request_error(error),
        }
    }
}

fn handle_request(
    socket: &mut UdpSocket,
    server: &mut server::DHCPServer,
) -> Result<(), RequestError> {
    // Read packet
    let mut buffer = [0; 576];
    let (packet_size, _) = match socket.recv_from(&mut buffer) {
        Ok(result) => result,
        Err(error) => return Err(RequestError::ReadRequestError(error)),
    };

    // Convert to correct size packet
    let buffer = &buffer[..packet_size];

    // Parse packet
    let packet = dhcp::DHCPPacket::parse(buffer)?;

    // Handle packet
    match server.handle_packet(packet)? {
        Some((response_packet, target)) => {
            match socket.send_to(
                response_packet.generate().as_slice(),
                match target {
                    Some(target) => target,
                    None => BROADCAST_ADDRESS.to_socket_addr(server::DHCP_CLIENT_PORT),
                },
            ) {
                Ok(_) => {}
                Err(error) => return Err(RequestError::WriteResponseError(error)),
            }
        }
        None => {}
    }

    Ok(())
}

impl std::error::Error for RuntimeError {}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RuntimeError::CreateServerError(error) =>
                    format!("Unable to create server ({})", error),
                RuntimeError::LoadConfigurationError(error) =>
                    format!("Error while loading configuration - {}", error),
            }
        )
    }
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RequestError::ReadRequestError(error) =>
                    format!("Unable to read request ({})", error),
                RequestError::ParsePacketError(error) =>
                    format!("Unable to parse packet ({})", error),
                RequestError::HandlePacketError(error) =>
                    format!("Error while handling packet - {}", error),
                RequestError::WriteResponseError(error) =>
                    format!("Unable to write response ({})", error),
            }
        )
    }
}

impl From<dhcp::PacketParseError> for RequestError {
    fn from(error: dhcp::PacketParseError) -> Self {
        RequestError::ParsePacketError(error)
    }
}

impl From<server::HandlePacketError> for RequestError {
    fn from(error: server::HandlePacketError) -> Self {
        RequestError::HandlePacketError(error)
    }
}

impl From<config::ConfigurationError> for RuntimeError {
    fn from(error: config::ConfigurationError) -> Self {
        RuntimeError::LoadConfigurationError(error)
    }
}
