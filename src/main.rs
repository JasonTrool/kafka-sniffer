use std::io::Read;
use std::str::from_utf8;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use stream::kafka::CorrelationMap;
use std::net::TcpStream;

#[derive(Debug)]
pub struct KafkaConnection {
    stream: TcpStream,
}

fn main() {
    // call kafka_sniffer_pcap::runner() or kafka_sniffer_smoltcp::runner()
}

mod kafka_sniffer_pcap;
mod kafka_sniffer_pcap_sys;
mod stream;
mod parser;
mod protocols;