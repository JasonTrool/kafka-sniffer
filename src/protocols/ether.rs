use crate::protocols::ipv4;

use serde::Serialize;


#[derive(Debug, PartialEq, Serialize)]
pub enum Ether {
    IPv4(pktparse::ipv4::IPv4Header, ipv4::IPv4),
    Unknown(Vec<u8>)
}