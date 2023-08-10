use core::fmt::{Debug, Display, Formatter, Result};

#[repr(C)]
pub struct Ipv4Address([u8; 4]);

impl Ipv4Address {
    pub fn new(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }
}

impl Debug for Ipv4Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "IpAddress({})", self)
    }
}

impl Display for Ipv4Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

#[repr(packed)]
pub struct Ipv4Header {
    version_ihl: u8,
    dscp_ecn: u8,
    total_length: u16,
    identification: u16,
    flags_fragment_offset: u16,
    ttl: u8,
    protocol: u8,
    header_checksum: u16,
    source: Ipv4Address,
    destination: Ipv4Address,
}

impl Ipv4Header {
    pub fn new(source: Ipv4Address, destination: Ipv4Address, protocol: Ipv4Protocol, data: &[u8]) -> Self {
        let total_length = (data.len() + 20) as u16;
        let identification = 0;
        let flags = 0;
        let fragment_offset = 0;

        let mut header = Self {
            version_ihl: 0x45,
            dscp_ecn: 0,
            total_length,
            identification,
            flags_fragment_offset: (flags << 13) | fragment_offset,
            ttl: 64,
            protocol: protocol as u8,
            header_checksum: 0,
            source,
            destination,
        };

        header.compute_checksum(data);

        header
    }

    fn compute_checksum(&mut self, data: &[u8]) {
        todo!()
    }
}

impl super::Header for Ipv4Header {
    fn compute_checksum(&mut self, data: &[u8]) {
        todo!()
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Ipv4Protocol {
    Icmp = 1,
    Tcp = 6,
    Udp = 17,
}