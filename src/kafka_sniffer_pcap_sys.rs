use anyhow::{Result};
use std::ffi::CString;
use std::ffi::CStr;
use std::mem;


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
        let status = pcap_sys::pcap_compile(self.handle, &mut bpf_program, filter.as_ptr() as *const i8, 1, pcap_sys::PCAP_NETMASK_UNKNOWN);
        if status != 0 {
            let err = self.get_error();
            eprintln!("at set_filter: {}", err);
        }

        // apply filter
        let status = pcap_sys::pcap_setfilter(self.handle, &mut bpf_program);
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
    let mut iface = "en0".to_owned();

    let _: Vec<String> = go_flag::parse(|flags| {
        flags.add_flag("i", &mut iface);
    });

    let conf = Config { 
        promisc: true,
        immediate_mode: true,
    };

    let mut cap = open(iface.as_str(), &conf).expect("unknown interface!!!");
    cap.set_filter("tcp port 9092")
    while let Ok(Some(packet)) = cap.next_packet() {
        println!("packet recvd: {:?}", packet.data);
    }
}

pub struct Packet {
    data: Vec<u8>,
}
