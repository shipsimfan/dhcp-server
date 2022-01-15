use std::net::UdpSocket;

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

fn log_formatter(record: &logging::Record) -> String {
    format!(
        "{} | {} | {} | {}",
        record.level(),
        record.timestamp().to_rfc2822(),
        record.name(),
        record.message()
    )
}

fn main() {
    // Prepare logging formatter & console output for early errors
    {
        let root_logger = logging::get_logger("");
        root_logger.remove_handler(0);
        root_logger.set_level(Some(logging::LogLevel::Informational));

        let mut handler = logging::Handler::new(logging::ConsoleHandler::new());
        handler.set_formatter(Some(log_formatter));

        root_logger.add_handler(handler);
    }

    match run() {
        Ok(()) => {}
        Err(error) => {
            let logger = logging::get_logger(module_path!());
            logging::critical!(logger, "{}", error);
        }
    }
}

fn run() -> Result<(), RuntimeError> {
    let logger = logging::get_logger(module_path!());

    // Load configuration
    let configuration = config::load_configuration()?;
    logging::info!(logger, "Configuration loaded");

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

    logging::info!(
        logger,
        "Server listening on port {}",
        server::DHCP_SERVER_PORT
    );

    // Handle requests
    loop {
        match handle_request(&mut socket, &mut server) {
            Ok(()) => {}
            Err(error) => logging::error!(logger, "{}", error),
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
