pub enum MessageType {
    Request,
    Reply,
}

impl MessageType {
    pub fn parse(value: u8) -> Option<Self> {
        match value {
            1 => Some(MessageType::Request),
            2 => Some(MessageType::Reply),
            _ => None,
        }
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MessageType::Request => "Request",
                MessageType::Reply => "Reply",
            }
        )
    }
}
