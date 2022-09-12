
use crate::link::{DataLink};

pub enum ParsedPacket {
}

pub fn parse(link: &DataLink, data: &[u8]) {
    match *link {
        DataLink::Ethernet => (), // TODO(threadedstream): handle ethernet packet
        DataLink::Tun => () // TODO(threadedstream): handle IPv4 or IPv6 packet
    }
}

pub fn parse_eth(data: &[u8]){
}

pub fn parse_tun(data: &[u8]) {
}