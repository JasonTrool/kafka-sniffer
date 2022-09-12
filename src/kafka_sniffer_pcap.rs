use pcap::{Device, Capture};

const DEFAULT_LISTEN_ADDR: &str  = ":9870";
const DEFAULT_EXPIRE_TIME_SECS: u64 = 5 * 60; // 5 minutes

pub fn runner() {
    let mut iface = "eth0".to_owned();
    let mut port = "9092".to_owned();
    let mut snaplen = 16 << 10;
    let filter = "tcp and port ".to_owned() + port.as_str();
    let mut verbose = false;
    let mut listen_addr = DEFAULT_LISTEN_ADDR.to_owned();
    let mut expire_time = DEFAULT_EXPIRE_TIME_SECS;

    let _: Vec<String> = go_flag::parse(|flags| {
        flags.add_flag("i", &mut iface);
        flags.add_flag("p", &mut port);
        flags.add_flag("s", &mut snaplen);
        flags.add_flag("v", &mut verbose);
        flags.add_flag("addr", &mut listen_addr);
        flags.add_flag("expire_time", &mut expire_time)
    });


    // haven't figured out how to specify an interface yet
    // tried Device::from, but it resulted in panic
    // so I roll with a default (perhaps, eth0 on my machine) device
    // TODO(threadedstream): FIGURE THAT OUT!
    let dev = Device::lookup().unwrap().unwrap();
    
    // it also panics with a timeout set to -10 ms
    let mut cap = Capture::from_device(dev).unwrap()
        .snaplen(snaplen as i32)
        .promisc(false)
        .open()
        .unwrap();
    
    cap.filter(filter.as_str(), true).expect("incorrect filter program");

    while let Ok(packet) = cap.next_packet() {  
        println!("packet received: {:?}", packet.data);
    }
}
