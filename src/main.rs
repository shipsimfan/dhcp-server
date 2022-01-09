use std::{net::UdpSocket, process::exit};

mod config;
mod dhcp;
mod server;

#[derive(Debug)]
enum RuntimeError {
    CreateServerError(std::io::Error),
}

#[derive(Debug)]
enum RequestError {
    ReadRequestError(std::io::Error),
    ParsePacketError,
}

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
    // Create DHCP Server
    let mut server = server::DHCPServer::new();

    // Create UDP Server
    let mut socket = match UdpSocket::bind("0.0.0.0:67") {
        Ok(socket) => socket,
        Err(error) => return Err(RuntimeError::CreateServerError(error)),
    };

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
    let (packet_size, source) = match socket.recv_from(&mut buffer) {
        Ok(result) => result,
        Err(error) => return Err(RequestError::ReadRequestError(error)),
    };

    // Convert to correct size packet
    let mut packet = Vec::with_capacity(packet_size);
    for i in 0..packet_size {
        packet.push(buffer[i]);
    }

    // Parse packet
    let packet = match dhcp::DHCPPacket::parse(packet.as_slice()) {
        Ok(packet) => packet,
        Err(()) => return Err(RequestError::ParsePacketError),
    };

    println!("Packet recieved from {}", source);

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
                RequestError::ParsePacketError => format!("Unable to parse packet"),
            }
        )
    }
}
