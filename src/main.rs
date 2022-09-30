#[macro_use]
extern crate lazy_static;

use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use crate::decoder::PacketDecoder;

fn main() {
    let sample = vec![0x20, 0x10, 0x01, 0x02, 0xA, 0xB];
    let mut decoder = decoder::RealDecoder::new(sample);
    let first = decoder.get_int32().unwrap();
    let second = decoder.get_int16().unwrap();
    println!("first => {}", first);
    println!("second => {}", second);
}


pub struct KafkaConnection {
    stream: TcpStream,
}

mod kafka_sniffer_pcap;
mod kafka_sniffer_pcap_sys;
mod parser;
mod stream;
mod protocols;
mod decoder;