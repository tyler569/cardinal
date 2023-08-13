mod ethernet;
mod icmp;
mod ipv4;
mod packet;

pub use ethernet::EthernetHeader;
pub use ethernet::MacAddress;
pub use icmp::IcmpHeader;
pub use ipv4::Ipv4Address;
pub use ipv4::Ipv4Header;
pub use ipv4::Ipv4Protocol;
pub use packet::Packet;

pub trait Header: Sized {
    fn compute_checksum(&mut self, data: &[u8]);

    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}
