use pktparse::{ethernet, ipv4};
use pktparse::ip::IPProtocol;
use pktparse::ethernet::EtherType;

use crate::protocols::link::{DataLink};
use crate::protocols::ipv4::IPv4;
use crate::protocols::ether::Ether;


pub enum Packet {
    EthernetPacket(Ether),
    IPv4Packet(IPv4)
}

// pub fn parse(link: &DataLink, data: &[u8]) -> Result<Packet>{
//     match *link {
//         DataLink::Tun => {
//             match parse_tun(data) {
//                 Ok((tun_packet)) => Ok(Packet(tun_packet)),
//                 Err(_) => Err("todo!"),
//             }
//         }
//         DataLink::Ethernet => {
//             match parse_eth(data) { 
//                 Ok((eth_packet)) => Ok(Packet(eth_packet)),
//                 Err(_) => Err("todo!"),
//             }
//         }
//     }
// }

// pub fn parse_eth(data: &[u8]) -> Result<Ether> {
//     if let Ok((remaining, eth_frame)) = ethernet::parse_ethernet_frame(data) {
//         let inner = match eth_frame.ethertype {
//             EtherType::IPv4 => match parse_ipv4()
//         }
//     }
// }

// pub fn parse_tun(data: &[u8]) -> Result<IPv4>{

// }

// fn parse_ipv4() -> Result<Ether>{
//     use crate::protocols::ipv4::IPv4::*;

//     if let Ok((remaining, ip_hdr)) = ipv4::parse_ipv4_header(data) {
//         let inner = match ip_hdr.protocol {
//             IPProtocol::TCP => match tcp::parse(remaining) {
//                 Ok((tcp_hdr, tcp)) => Ok(TCP(tcp_hdr, tcp)),
//                 Err(_) => Ok(Unknown(remaining.to_vec()))
//             }
//             _ => unreachable!()
//         };
//     }
//     Err("todo!")
// }