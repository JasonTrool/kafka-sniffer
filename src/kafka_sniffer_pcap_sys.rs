use anyhow::{Result};
use kafka_protocol::protocol::buf::ByteBuf;
use std::ffi::CString;
use std::ffi::CStr;
use std::mem;
use kafka_protocol::messages::{RequestHeader, ApiVersionsRequest, ApiKey, RequestKind};
use kafka_protocol::messages;
use kafka_protocol::protocol::{Encodable, Decodable, StrBytes};
use bytes::{BytesMut, Buf};
use std::{thread, time};
use serde::{Deserializer, Deserialize};

pub struct Cap {
    handle: *mut pcap_sys::pcap,
}

pub struct Config {
    pub promisc: bool,
    pub immediate_mode: bool,
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
    let data = vec![0x2, 0x0, 0x0, 0x0, 0x45, 0x0,
                    0x0, 0x70, 0x0, 0x0, 0x40, 0x0,
                    0x40, 0x06, 0x0, 0x0, 0xc0, 0xa8,
                    0x00, 0xc7, 0xc0, 0xa8, 0x00, 0xc7,
                    0xdf, 0xf5, 0x23, 0x84, 0x19, 0x36,
                    0xe1, 0x77, 0xdf, 0x6c, 0x7e, 0x5a,
                    0x80, 0x18, 0x95, 0xb4, 0x83, 0x41,
                    0x00, 0x00, 0x01, 0x01, 0x08, 0x0a,
                    0xde, 0xa1, 0xfc, 0x6d, 0x24, 0x2c,
                    0x5f, 0x36, 0x00, 0x00, 0x00, 0x38,
                    0x00, 0x01, 0x00, 0x0d, 0x00, 0x00,
                    0x07, 0xa6, 0x00, 0x10, 0x63, 0x6f,
                    0x6e, 0x73, 0x6f, 0x6c, 0x65, 0x2d,
                    0x63, 0x6f, 0x6e, 0x73, 0x75, 0x6d,
                    0x65, 0x72, 0x00, 0xff, 0xff, 0xff,
                    0xff, 0x00, 0x00, 0x01, 0xf4, 0x00,
                    0x00, 0x00, 0x01, 0x03, 0x20, 0x00,
                    0x00, 0x00, 0x5e, 0xcc, 0x4b, 0x6a,
                    0x00, 0x00, 0x06, 0x80, 0x01, 0x01,
                    0x01, 0x00];

    decode_packet(&data);
    while let Ok(Some(packet)) = cap.next_packet() {
        decode_packet(&packet.data);
        thread::sleep(duration);
    }
}

fn decode_packet(data: &[u8]) {
    let mut buf = BytesMut::from(data);
    let family = buf.peek_bytes(0..4).get_i16_le();
    println!("we're like a family! NO! Brothers and sisters => {}", family);
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
    let mut kafka_data = match pktparse::tcp::parse_tcp_header(tcp_header.as_slice()) {
        Ok((data, header)) => {
            println!("source port => {}", header.source_port);
            println!("dest port => {}", header.dest_port);
            data
        }
        Err(_) => {
            panic!("failed to parse tcp header");
        }
    };


    let message_length = kafka_data.peek_bytes(0..4).get_i32();
    let api_key = ApiKey::try_from(kafka_data.peek_bytes(4..6).get_i16()).unwrap();
    let version = kafka_data.peek_bytes(6..8).get_i16();
    match api_key {
        ApiKey::FetchKey => {
            let mut fetch_req_raw = kafka_data.peek_bytes(8usize..message_length as usize);
            let req = kafka_protocol::messages::FetchRequest::decode(&mut fetch_req_raw, version).unwrap();
            for topic in &req.topics {
                println!("{}", topic.topic.to_string())
            }
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
