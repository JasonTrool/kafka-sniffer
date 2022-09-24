use serde::Serialize;


#[derive(Debug, PartialEq, Serialize)]
pub enum UDP {
    Text(String),
    Binary(Vec<u8>),
}



