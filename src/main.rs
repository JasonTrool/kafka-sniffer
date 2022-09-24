use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use stream::kafka::CorrelationMap;


fn main() {
    // run kafka_sniffer_pcap::runner() or kafka_sniffer_smoltcp::runner()
    kafka_sniffer_pcap_sys::runner();
}

fn setup_kafka_connection() {
}

pub enum KafkaStream {
    
}

pub struct KafkaConnection {
    stream: TcpStream,
}

mod kafka_sniffer_pcap;
mod kafka_sniffer_pcap_sys;
mod stream;
mod parser;
mod protocols;