#[repr(packed)]
#[derive(Debug)]
pub struct IcmpHeader {
    pub icmp_type: IcmpType,
    pub icmp_code: u8,
    pub checksum: u16,
    pub rest: [u8; 4],
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum IcmpType {
    EchoReply = 0,
    EchoRequest = 8,
}

impl IcmpHeader {
    pub fn new_echo_request(data: &[u8]) -> Self {
        let mut header = Self {
            icmp_type: IcmpType::EchoRequest,
            icmp_code: 0,
            checksum: 0,
            rest: [0; 4],
        };

        header
    }
}

impl super::Header for IcmpHeader {
    fn compute_checksum(&mut self, data: &[u8]) {
        todo!()
    }
}