use std::str::from_utf8;
use anyhow::{Result, Error};
use pktparse::tcp::{self, TcpHeader};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub enum TCP {
    // TODO(threadedstream): tls
    Text(String),
    Binary(Vec<u8>),
    Empty,
}

// TODO(threadedstream): it had a lot of errors, comment that out for a moment
// pub fn parse(data: &[u8]) -> Result<(tcp::TcpHeader, TCP)> {
//     if let Ok((remaining, tcp_hdr)) = tcp::parse_tcp_header(data) {
//         let inner = match extract(&tcp_hdr, remaining) {
//             Ok(x) => Ok((tcp_hdr, x)),
//             Err(_) => Err("todo!"),
//         };
//     }
// }

#[inline]
pub fn extract(_tcp_hdr: &TcpHeader, data: &[u8]) -> Result<TCP> {
    if data.is_empty() {
        Ok(TCP::Empty)
    } else if data.contains(&0) {
        Ok(TCP::Binary(data.to_vec()))     
    } else {
        match from_utf8(data) {
            Ok(data) => Ok(TCP::Text(data.to_owned())),
            Err(_) =>  Err(anyhow::Error::msg("todo!"))
        }
    }
}

