use alloc::vec::Vec;

pub struct Packet {
    pub data: Vec<u8>,
}

impl Packet {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: Vec::from(data),
        }
    }
}
