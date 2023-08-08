use core::fmt::{Debug, Display};

pub struct MacAddress {
    bytes: [u8; 6],
}

impl MacAddress {
    pub fn new(bytes: [u8; 6]) -> Self {
        Self {
            bytes,
        }
    }
}

impl Debug for MacAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MacAddress({})", self)
    }
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.bytes[0], self.bytes[1], self.bytes[2],
            self.bytes[3], self.bytes[4], self.bytes[5],
        )
    }
}