use crate::protocols::tcp;

use serde::Serialize;

#[derive(Debug, PartialEq)]
pub enum IPv4 {
    TCP(pktparse::tcp::TcpHeader, tcp::TCP)
}