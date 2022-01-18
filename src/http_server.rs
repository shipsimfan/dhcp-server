use std::{
    collections::VecDeque,
    sync::{Mutex, Once},
};

struct ServerContainer(Option<Mutex<Server>>);

struct ServerHandler(&'static Mutex<Server>);

struct Server {
    configuration_body: String,
    logs: VecDeque<(String, logging::LogLevel, String, String)>,
    log_limit: Option<usize>,
}

fn log_client_error(error: http::HandleClientError) {
    logging::error!(
        logging::get_logger(module_path!()),
        "Client Error - {}",
        error
    );
}

fn log_server_error(error: std::io::Error) {
    logging::critical!(
        logging::get_logger(module_path!()),
        "Error while starting HTTP server - {}",
        error
    );
}

static SERVER_INIT: Once = Once::new();
static mut SERVER: ServerContainer = ServerContainer(None);

pub fn start(configuration: &crate::config::Configuration) {
    // Generate the configuration body
    let mut body = format!("<html>");
    body.push_str("<head>");
    body.push_str("<title>DHCP Server</title>");
    body.push_str("<meta name='viewport' content='width=device-width, initial-scale=1.0'>");
    body.push_str("<link rel='stylesheet' href='/style' />");
    body.push_str("<link rel='preconnect' href='https://fonts.googleapis.com'><link rel='preconnect' href='https://fonts.gstatic.com' crossorigin><link href='https://fonts.googleapis.com/css2?family=Roboto&display=swap' rel='stylesheet'>");
    body.push_str("</head>");

    body.push_str("<body>");
    body.push_str("<h1>DHCP Server</h1>");
    body.push_str("<h2>Configuration</h2>");
    body.push_str("<h3>Lease Configuration</h3>");
    body.push_str(&format!(
        "<b>Starting IP Address:</b> {}<br />",
        configuration.lease_start_ip()
    ));
    body.push_str(&format!(
        "<b>Final IP Address:</b> {}<br />",
        configuration.lease_final_ip()
    ));
    body.push_str(&format!(
        "<b>Lease Time:</b> {} seconds<br />",
        configuration.address_time()
    ));
    body.push_str(&format!(
        "<b>Renewal Time:</b> {} seconds<br />",
        configuration.renewal_time()
    ));
    body.push_str(&format!(
        "<b>Rebinding Time:</b> {} seconds<br />",
        configuration.rebinding_time()
    ));
    body.push_str(&format!(
        "<b>Offer Time:</b> {} seconds<br />",
        configuration.offer_time()
    ));
    body.push_str("<h3>Network Configuration</h3>");
    body.push_str(&format!(
        "<b>Our IP Address:</b> {}<br />",
        configuration.our_ip()
    ));
    body.push_str(&format!(
        "<b>Gateway IP Address:</b> {}<br />",
        configuration.gateway_ip()
    ));
    body.push_str(&format!(
        "<b>Subnet Mask:</b> {}<br />",
        configuration.subnet_mask()
    ));
    body.push_str(&format!(
        "<b>Broadcast IP Address:</b> {}<br />",
        configuration.broadcast_address()
    ));
    let (dns, dns_alternate) = configuration.dns();
    body.push_str(&format!("<b>D.N.S. Server:</b> {}<br />", dns));
    body.push_str(&format!(
        "<b>D.N.S. Alternative Server:</b> {}<br />",
        dns_alternate
    ));
    body.push_str("<h2>Allocated IP Addresses</h2>");
    body.push_str("<h3>Reserved IP Addresses</h3>");
    if configuration.reserved_ips().len() > 0 {
        body.push_str("<table>");
        body.push_str("<tr><th>IP Address</th><th>MAC Address</th></tr>");
        for (mac, ip) in configuration.reserved_ips() {
            body.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", ip, mac));
        }
        body.push_str("</table>");
    } else {
        body.push_str("No IP addresses have been reserved");
    }

    // Create HTTP Server
    SERVER_INIT.call_once(|| unsafe {
        SERVER = ServerContainer(Some(Mutex::new(Server {
            configuration_body: body,
            logs: VecDeque::new(),
            log_limit: configuration.log_limit(),
        })))
    });

    // Setup logging handler
    {
        let output = ServerHandler(unsafe { SERVER.0.as_ref() }.unwrap());
        let handler = logging::Handler::new(Box::new(output));

        let root_logger = logging::get_logger("");
        root_logger.add_handler(handler);
    }

    std::thread::spawn(|| {
        match http::start_server(80, unsafe { &SERVER }, Some(log_client_error)) {
            Ok(()) => {}
            Err(error) => log_server_error(error),
        };
    });
}

impl Server {
    pub fn handle_request(&self) -> http::Response {
        let mut body = self.configuration_body.clone();

        // Append current leases
        let leases = unsafe { crate::DHCP_SERVER.as_ref() }
            .unwrap()
            .lock()
            .unwrap()
            .current_leases();

        body.push_str("<h3>Leased IP Addresses</h3>");
        if leases.len() > 0 {
            body.push_str("<table>");
            body.push_str("<tr><th>IP Address</th><th>MAC Address</th></tr>");
            for (ip, mac) in leases {
                body.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", ip, mac));
            }
            body.push_str("</table>");
        } else {
            body.push_str("No IP addresses are currently leased");
        }

        // Append logs
        body.push_str("<h2>Log</h2>");
        if self.logs.len() > 0 {
            body.push_str("<table>");
            body.push_str(
                "<tr><th>Date & Time</th><th>Severity</th><th>Source</th><th>Message</th></tr>",
            );
            for (timestamp, level, name, message) in &self.logs {
                body.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    timestamp, level, name, message
                ));
            }
            body.push_str("</table>");
        } else {
            body.push_str("No logs have been recorded");
        }

        body.push_str("</body>");
        body.push_str("</html>");

        let mut response = http::Response::new_status(http::Status::Ok, Some(body));
        response
            .header_mut()
            .insert_header(format!("Content-Type"), format!("text/html"));

        response
    }

    pub fn push_log(&mut self, record: &logging::Record) {
        self.logs.push_front((
            record.timestamp().to_rfc2822(),
            record.level(),
            record.name().to_owned(),
            record.message().to_owned(),
        ));

        match self.log_limit {
            Some(limit) => {
                if self.logs.len() > limit {
                    self.logs.pop_back();
                }
            }
            None => {}
        }
    }
}

impl http::Server for ServerContainer {
    fn handle_request(&self, request: http::Request) -> http::Response {
        match request.header().method() {
            http::Method::Get => {
                if request.header().uri() == "/style" {
                    let mut response = http::Response::new_status(
                        http::Status::Ok,
                        Some(include_str!("./style.css").to_owned()),
                    );
                    response
                        .header_mut()
                        .insert_header(format!("Content-Type"), format!("text/css"));
                    response
                } else {
                    self.0.as_ref().unwrap().lock().unwrap().handle_request()
                }
            }
            _ => http::Response::new_status(http::Status::NotFound, None),
        }
    }
}

impl logging::HandlerType for ServerHandler {
    fn emit(&mut self, record: &logging::Record, _: logging::Formatter) {
        self.0.lock().unwrap().push_log(record);
    }

    fn flush(&mut self) {}
}
