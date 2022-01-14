use std::net::SocketAddr;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IPAddress([u8; 4]);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MACAddress([u8; 6]);

impl IPAddress {
    pub const fn new(address: [u8; 4]) -> Self {
        IPAddress(address)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn to_socket_addr(&self, port: u16) -> SocketAddr {
        SocketAddr::from((self.0, port))
    }

    pub fn increament(&mut self) {
        // Increament but ignore address's that satisfy one of the following conditions:
        //  1 - x.x.x.255       (May be a broadcast address)
        //  2 - x.x.255.255     (Same as above)
        //  3 - x.255.255.255   (Same as above)
        //  4 - 255.255.255.255 (Is broadcast address)
        //  5 - x.x.x.0         (Some programs don't like client addresses ending in 0)

        if self.0[3] == 255 {
            self.0[3] = 0;
            if self.0[2] == 255 {
                self.0[2] = 0;
                if self.0[1] == 255 {
                    self.0[1] = 0;
                    if self.0[0] == 255 {
                        panic!("Overflow on increamenting I.P. Address");
                    } else {
                        self.0[0] += 1;
                    }
                } else {
                    self.0[1] += 1;
                }
            } else {
                self.0[2] += 1;
            }
        } else {
            self.0[3] += 1;
        }

        if self.0[3] == 255 || self.0[3] == 0 {
            self.increament();
        }
    }
}

impl std::fmt::Display for IPAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl std::cmp::PartialOrd for IPAddress {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for IPAddress {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.0[0].cmp(&other.0[0]) {
            std::cmp::Ordering::Equal => match self.0[1].cmp(&other.0[1]) {
                std::cmp::Ordering::Equal => match self.0[2].cmp(&other.0[2]) {
                    std::cmp::Ordering::Equal => self.0[3].cmp(&other.0[3]),
                    std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
                    std::cmp::Ordering::Less => std::cmp::Ordering::Less,
                },
                std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
                std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            },
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
        }
    }
}

impl MACAddress {
    pub const fn new(address: [u8; 6]) -> Self {
        MACAddress(address)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for MACAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}:{}:{}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}
