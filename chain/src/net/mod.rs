pub mod messages;

use {
    crate::net::messages::Message,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IpAddress {
    V4([u8; 4]),
    V6([u8; 16]),
}

pub trait PeerTrait: Send + Sync {
    fn get_ip_address(&self) -> IpAddress;
    fn send_message(&self, message: Message);
    fn receive_message(&self) -> Option<Message>;
}

impl std::fmt::Display for IpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpAddress::V4(ip) => write!(f, "{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]),
            IpAddress::V6(ip) => {
                let groups: Vec<u16> = ip
                    .chunks(2)
                    .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                    .collect();
                write!(f, "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}", 
                    groups[0], groups[1], groups[2], groups[3], 
                    groups[4], groups[5], groups[6], groups[7])
            }
        }
    }
}