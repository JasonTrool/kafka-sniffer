
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub enum TCP {
    Binary(Vec<u8>),
    Empty,
}