mod packet;
mod proto;
pub mod socket;

pub use packet::Packet;
pub use socket::Socket;

pub use proto::ethernet::MacAddress;

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
