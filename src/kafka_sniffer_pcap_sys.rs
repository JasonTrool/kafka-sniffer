use anyhow::{Result};
use kafka_protocol::protocol::buf::ByteBuf;
use lazy_static::lazy_static;
use std::f32::consts::E;
use std::ffi::CString;
use std::ffi::CStr;
use std::mem;
use kafka_protocol::messages::{RequestHeader, ApiKey};
use kafka_protocol::messages;
use kafka_protocol::protocol::{Decodable};
use bytes::{BytesMut, Bytes, Buf};
use std::collections::HashMap;
use std::{thread, time};
use std::sync::{Mutex};
use std::error;
use std::time::{SystemTime, Duration};
use std::fmt;
use std::fmt::Formatter;

use crate::stream::kafka::{CorrelationMap};

const MAX_RESPONSE_SIZE: i32 = 10 * 1024 * 1024; // 10MB
const MIN_RESPONSE_SIZE: i32 = 5;

lazy_static! {
    static ref CORRELATION_MAP: Mutex<CorrelationMap> = Mutex::new(CorrelationMap::new());
}

type KafkaSnifferResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
pub struct InvalidLengthValueError;

impl fmt::Display for InvalidLengthValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid value of length")
    }
}

#[derive(Debug, Clone)]
pub struct FailedToGetKeyAndVersionError;

impl fmt::Display for FailedToGetKeyAndVersionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to get key and version using correlationID")
    }
}

#[derive(Debug, Clone)]
pub struct UnsupportedBodyWithKeyError;

impl fmt::Display for UnsupportedBodyWithKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsupported body with key")
    }
}

impl error::Error for InvalidLengthValueError {} 
impl error::Error for FailedToGetKeyAndVersionError {}
impl error::Error for UnsupportedBodyWithKeyError {}

pub struct Cap {
    handle: *mut pcap_sys::pcap,
}

pub struct Config {
    pub promisc: bool,
    pub immediate_mode: bool,
}

pub struct KafkaResponse {
    pub key: i16,
    pub version: i16,
    pub body_length: i32,
    pub correlation_id: i32,
    pub client_id: Option<String>,
    pub body: Box<dyn ProtocolBody>,
    pub use_prepared_key_version: bool,
}

impl KafkaResponse {
    pub fn decode<T: ByteBuf>(&self, data: &mut T, version: i16) -> KafkaSnifferResult<Self> {
        let length = data.peek_bytes(0..4).get_i32();
        let correlation_id = data.peek_bytes(4..8).get_i32();
        if length < MIN_RESPONSE_SIZE || length > MAX_RESPONSE_SIZE {
            return Err(InvalidLengthValueError.into());
        }
    
        // use correlation map to pull key and version out
        let mut key: i16;
        let mut version: i16;
        if let Some(value) = CORRELATION_MAP.lock().unwrap().get_and_remove(correlation_id) {
            key = value[0];
            version = value[1];
        } else {
            return Err(FailedToGetKeyAndVersionError.into());
        }
        
        let remaining = data.peek_bytes(8..4+length as usize).to_vec();

        todo!()
        // if key == 0 {
        //     self.body = Box::new(ProduceResponse::decode(&Bytes::from(remaining), version).unwrap());
        // } else if key == 1 {
        //     //self.body = Box::new(FetchResponse::decode())
        // }

        // Ok(KafkaResponse {
        //     key: key,
        //     version: version,
        //     body_length: length,
        //     correlation_id,
        //     client_id: None,
        //     body: 
        //     use_prepared_key_version: true,
        // })
    }

    fn allocate_response_body(&self, key: i16, version: i16) -> Option<Box<dyn ProtocolBody>> {
        match key {
            0 => Some(Box::new(ProduceResponse::new())),
            1 => Some(Box::new(FetchResponse::new())),
            _ => None,
        }
    }
}


pub trait ProtocolBody{
    fn key(&self) -> i16;
    fn version(&self) -> i16;
}

#[derive(Debug)]
pub struct ProduceResponseBlock {
    pub err: i16,
    pub offset: i64,
    pub timestamp: SystemTime,
    pub start_offset: i64,
}

#[derive(Debug)]
pub struct ProduceResponse {
    pub blocks: HashMap<String, HashMap<i32, Option<ProduceResponseBlock>>>,
    pub version: i16,
    pub throttle_time: Duration,
}

impl ProduceResponse {
    pub fn new() -> Self {
        ProduceResponse {
            version: 0,
            throttle_time: Duration::default(),
            blocks: HashMap::new(),
        }
    }

    pub fn decode<T: ByteBuf>(data: &T, version: i16) -> KafkaSnifferResult<ProduceResponse> {
        let mut response = ProduceResponse::new();
        response.version = version; 
        todo!()
    }
}


impl ProtocolBody for ProduceResponse {
    fn key(&self) -> i16 {
        0
    }

    fn version(&self) -> i16 {
        self.version
    }

}

#[derive(Debug)]
pub struct FetchResponse {
    pub throttle_time: Duration,
    pub error_code: i16,
    pub session_id: i32,
    pub version: i16,
    pub log_append_time: bool,
    pub timestamp: SystemTime,
}

impl FetchResponse {
    pub fn new() -> Self {
        FetchResponse {
            throttle_time: Duration::default(),
            error_code: 0,
            session_id: 0,
            version: 0,
            log_append_time: false,
            timestamp: SystemTime::now(),
        }
    }

    pub fn decode<T: ByteBuf>(data: &T, version: i16) -> KafkaSnifferResult<FetchResponse> {
        todo!()
    }
}

impl ProtocolBody for FetchResponse {
    fn key(&self) -> i16 {
        1
    }
    
    fn version(&self) -> i16 {
        self.version
    }
}

#[derive(Debug)]
pub struct Message {
    pub codec: i8, 
    pub compression_level: i32,
    pub log_append_time: bool,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub set: MessageSet,
    pub version: i8,
    pub timestamp: SystemTime,
}

#[derive(Debug)]
pub struct MessageBlock {
    pub offset: i64,
    pub message: Option<Message>
}

#[derive(Debug)]
pub struct MessageSet {
    partial_trailing_message: bool,
    overflow_message: bool,
    messages: Vec<MessageBlock>,
}

#[derive(Debug)]
pub struct RecordBatch {
}

pub enum Records {
    MsgSet(MessageSet),
    RecordBatch(RecordBatch),
}

#[derive(Debug)]
pub struct ProduceRequest {
    pub transactional_id: Option<String>,
    pub required_acks: i16,
    pub timeout: i32,
    pub version: i16,
    pub records: HashMap<String, HashMap<i32, RecordBatch>>,
}

pub fn open(device: &str, config: &Config) -> Result<Cap>{
    let mut errbuf = [0; pcap_sys::PCAP_ERRBUF_SIZE as usize];
    let dev = CString::new(device).unwrap();
    let handle = unsafe { pcap_sys::pcap_create(dev.as_ptr(), errbuf.as_mut_ptr()) };
    if handle.is_null() {
        let err = unsafe { CStr::from_ptr(errbuf.as_ptr()) };
        panic!("Panic! At the open, reason: {:?}", err.to_str());
    }
    
    if config.promisc {
        unsafe { pcap_sys::pcap_set_promisc(handle, 1) };
    } 

    if config.immediate_mode {
        unsafe { pcap_sys::pcap_set_immediate_mode(handle, 1) };
    }

    let ret = unsafe { pcap_sys::pcap_activate(handle) };
    if ret != 0 {
        let err = unsafe { pcap_sys::pcap_geterr(handle) };
        let err = unsafe { CStr::from_ptr(err) };
        panic!("Failed to activate interface: {:?}", err.to_str())
    }

    Ok(Cap {
        handle,
    })
}


impl Cap {
    pub fn datalink(&self) -> i32 {
        unsafe { pcap_sys::pcap_datalink(self.handle) }
    }
    
    pub fn next_packet(&mut self) -> Result<Option<Packet>> {
        use std::mem::MaybeUninit;

        let mut header = MaybeUninit::<*mut pcap_sys::pcap_pkthdr>::uninit();
        let mut packet = MaybeUninit::<*const libc::c_uchar>::uninit();

        let retcode = unsafe { pcap_sys::pcap_next_ex(self.handle, header.as_mut_ptr(), packet.as_mut_ptr()) };

        match retcode {
            i if i >= 1 => {
                let header = unsafe { header.assume_init() };
                let packet = unsafe { packet.assume_init() };

                use std::slice;
                let packet = unsafe { slice::from_raw_parts(packet, (*header).caplen as _) };

                Ok(Some(Packet {
                    data: packet.to_vec()
                }))
            },
            0 => panic!("timeout expired"),
            -2 => Ok(None),
            _ => unreachable!(),
        }
    }

    pub unsafe fn set_filter(&self, filter: &str) {
        // compile a filter code
        let mut bpf_program: pcap_sys::bpf_program = mem::zeroed();
        let filter = CString::new(filter).unwrap();
        let status = pcap_sys::pcap_compile(self.handle, &mut bpf_program, filter.as_ptr(), 1, 0);
        if status != 0 {
            println!("status is {}", status);
            let err = self.get_error();
            eprintln!("at set_filter: {}", err);
        }

        // apply filter
        let status = pcap_sys::pcap_setfilter(self.handle, &mut bpf_program);
        pcap_sys::pcap_freecode(&mut bpf_program);
        if status != 0 {
            let err = self.get_error();
            eprintln!("at set_filter: {}", err);
        }
    }

    fn get_error(&self) -> String {
        let mut errbuf = [0; pcap_sys::PCAP_ERRBUF_SIZE as usize];
        let err = unsafe { pcap_sys::pcap_geterr(self.handle) };
        let err = unsafe { CStr::from_ptr(err) };
        err.to_str().unwrap().to_owned()
    }
}

// releasing pcap resources upon drop
impl Drop for Cap {
    fn drop(&mut self) {
        unsafe { pcap_sys::pcap_close(self.handle) };
    }
}

pub fn runner() {
    let mut iface = "lo0".to_owned();

    let _: Vec<String> = go_flag::parse(|flags| {
        flags.add_flag("i", &mut iface);
    });

    let conf = Config { 
        promisc: true,
        immediate_mode: true,
    };

    let mut cap = open(iface.as_str(), &conf).expect("unknown interface!!!");
    unsafe { cap.set_filter("tcp and port 9092") };
    let duration = time::Duration::from_secs(1);
    while let Ok(Some(packet)) = cap.next_packet() {
        decode_packet(&packet.data);
        thread::sleep(duration);
    }
}

fn decode_packet(data: &[u8]) {
    let mut buf = BytesMut::from(data);
    let family = buf.peek_bytes(0..4).get_i16_le();
    println!("family => {}", family);
    let ihl = buf.peek_bytes(4..5).get_i8();
    // get the size of a header
    let size = ((ihl & 0x0F) * 4) as usize;
    let ipv4_header = buf.peek_bytes(4..size+4).to_vec();
    match pktparse::ipv4::parse_ipv4_header(ipv4_header.as_slice()) {
        Ok((data, header)) => {
            println!("version => {}", header.version);
        }
        Err(_) => {
            eprintln!("failed to parse ipv4 header");
        }
    }

    let tcp_header = buf[size+4.. ].to_vec();
    let (mut kafka_data, tcp_header) = match pktparse::tcp::parse_tcp_header(tcp_header.as_slice()) {
        Ok((data, header)) => {
            println!("source port => {}", header.source_port);
            println!("dest port => {}", header.dest_port);
            (data, header)
        }
        Err(_) => {
            panic!("failed to parse tcp header");
        }
    };

    // how do we distinguish between response and request?
    // right, we look at the source and destination ports, if the former
    // evaluates to 9092, then we know for sure it's a response.

    let raw = kafka_data.to_vec();
    println!("{:02X?}", raw);
    if tcp_header.source_port == 9092 {
        // dealing with a response
        decode_kafka_response(&kafka_data);
    } else {
        decode_kafka_request(&kafka_data);
    }
}

fn decode_kafka_response(data: &[u8]) {
    //let mut res = KafkaResponse::decode(&mut Bytes::from(data.to_vec()), 13).unwrap();
    todo!()
}

fn decode_kafka_request(data: &[u8]) {
    let req = kafka_protocol::messages::RequestHeader::decode(&mut Bytes::from(data.to_vec()), 13);
        let api_key = ApiKey::try_from(0).unwrap();
        match api_key {
            ApiKey::FetchKey => {
                // let req = kafka_protocol::messages::FetchRequest::decode(&mut, version).unwrap();
                // for topic in &req.topics {
                //     println!("{}", topic.topic.to_string())
                // }
            }
            _ => {
                println!("unknown one")
            }
        }
}

fn decode_kafka_packet(data: &[u8]) {
    let mut buf = BytesMut::from(data);
    let version = buf.peek_bytes(2..4).get_i16();
    let header = RequestHeader::decode(&mut buf, version).unwrap();
    let api_key = ApiKey::try_from(header.request_api_version).unwrap();
    let req = match api_key {
        ApiKey::CreateTopicsKey => {
            println!("###### CreateTopicsKey ######");
            let req = messages::CreateTopicsRequest::decode(&mut buf, version);
            match req {
                Ok(topic_req) => {
                    topic_req.topics.into_iter().for_each(|(topic_name, creatable_topic)| {
                        println!("topic name => {}", topic_name.to_string());
                        println!("number of partitions => {}", creatable_topic.num_partitions);
                    });
                }
                Err(err) => {
                    eprintln!("err: {:?}", err);
                }
            }
        },
        ApiKey::FetchKey => println!("request to fetch"),
        ApiKey::ProduceKey => {
            println!("###### ProduceKey ######");
            let req = messages::ProduceRequest::decode(&mut buf, version);
            match req {
                Ok(produce_req) => {
                    println!("acks => {}", produce_req.acks);
                    println!("transactional_id => {:?}", produce_req.transactional_id);
                }
                Err(err) => {
                    eprintln!("{:?}", err);
                }
            }
        }
        _ => println!("unknown request with api key {:?}", api_key),
    };
}

pub struct Packet {
    data: Vec<u8>,
}
