use crate::protocols::tcp;
use crate::protocols::udp;

use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub enum IPv4 {
    TCP(pktparse::tcp::TcpHeader, tcp::TCP),
    UDP(pktparse::udp::UdpHeader, udp::UDP),
    Unknown(Vec<u8>)
}