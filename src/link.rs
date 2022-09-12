use anyhow::{Result};
use futures::future::UnwrapOrElse;

#[derive(Debug, Clone)]
pub enum DataLink {
    Ethernet,
    Tun,
}

impl DataLink {
    pub fn from_linktype(linktype: i32) -> Result<DataLink> {
        match linktype {
            1 => {
                Ok(DataLink::Ethernet)
            },
            12 => {
                Ok(DataLink::Tun)
            },
            _ => panic!("unknown linktype")
        }
    }
}