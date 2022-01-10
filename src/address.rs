#[derive(Clone, Copy)]
pub struct IPAddress([u8; 4]);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MACAdddress([u8; 6]);

impl IPAddress {
    pub const fn new(address: [u8; 4]) -> Self {
        IPAddress(address)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for IPAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl MACAdddress {
    pub const fn new(address: [u8; 6]) -> Self {
        MACAdddress(address)
    }
}

impl std::fmt::Display for MACAdddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}:{}:{}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}
