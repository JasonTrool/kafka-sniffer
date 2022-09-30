use std::error;
use std::fmt::{Display};
use bytes::{BytesMut, Buf};
use integer_encoding::VarInt;

type DecoderResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, PartialEq)]
pub struct ErrInsufficientData;

impl Display for ErrInsufficientData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "kafka: insufficient data to decode packet, more bytes expected")
    }
}

impl error::Error for ErrInsufficientData {} 

#[derive(Debug, Clone, PartialEq)]
pub struct ErrInvalidArrayLength;

impl Display for ErrInvalidArrayLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid array length")
    }
}

impl error::Error for ErrInvalidArrayLength {}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrInvalidByteSliceLength;

impl Display for ErrInvalidByteSliceLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid byteslice length")
    }
}

impl error::Error for ErrInvalidByteSliceLength {} 

#[derive(Debug, Clone, PartialEq)]
pub struct ErrInvalidStringLength;

impl Display for ErrInvalidStringLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid string length")
    }
}

impl error::Error for ErrInvalidStringLength {}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrVarintOverflow;

impl Display for ErrVarintOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "varint overflow")
    }
}

impl error::Error for ErrVarintOverflow {}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrInvalidBool;

impl Display for ErrInvalidBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid bool")
    }
}

impl error::Error for ErrInvalidBool {}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrDummy;

impl Display for ErrDummy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl error::Error for ErrDummy {}

pub trait PacketDecoder {
    fn get_int8(&mut self) -> DecoderResult<i8>; 
    fn get_int16(&mut self) -> DecoderResult<i16>;
    fn get_int32(&mut self) -> DecoderResult<i32>;
    fn get_int64(&mut self) -> DecoderResult<i64>;
    fn get_varint(&mut self) -> DecoderResult<i64>;
    fn get_array_length(&mut self) -> DecoderResult<u32>;
    fn get_bool(&mut self) -> DecoderResult<bool>;

    fn get_bytes(&mut self) -> DecoderResult<Vec<u8>>;
    fn get_varint_bytes(&mut self) -> DecoderResult<Vec<u8>>;
    fn get_raw_bytes(&mut self, length: i64) -> DecoderResult<Vec<u8>>;
    fn get_string(&mut self) -> DecoderResult<String>;
    fn get_nullable_string(&mut self) -> DecoderResult<Option<String>>;
    fn get_int32_array(&mut self) -> DecoderResult<Vec<i32>>;
    fn get_int64_array(&mut self) -> DecoderResult<Vec<i64>>;
    fn get_string_array(&mut self) -> DecoderResult<Vec<String>>;

    fn remaining(&mut self) -> i64;
    fn get_subset(&mut self, length: i64) -> DecoderResult<Box<dyn PacketDecoder>>;
    fn peek(&mut self, offset: i64, length: i64) -> DecoderResult<Box<dyn PacketDecoder>>;
    fn peek_int8(&mut self, offset: i64) -> DecoderResult<i8>;

    fn discard(&mut self, length: i64);

    fn push(&mut self, push_decoder: &mut dyn PushDecoder) -> Option<Box<dyn error::Error>>;
    fn pop(&mut self) -> Option<Box<dyn error::Error>>;
}

pub trait PushDecoder {
    fn save_offset(&mut self, i: i64);
    fn reserve_length(&mut self) -> i64;
    fn check(&mut self, curr_offset: i64, buf: Vec<u8>) -> Option<Box<dyn error::Error>>;
}

pub trait DynamicPushDecoder: PushDecoder + Decoder {}

pub trait Decoder {
    fn decode(&mut self, decoder: &dyn PacketDecoder) -> Option<Box<dyn error::Error>>;
}

pub trait VersionedDecoder {
    fn decode(&mut self, decoder: &dyn PacketDecoder, version: i16) -> Option<Box<dyn error::Error>>;
}

pub struct RealDecoder {
    pub raw: Vec<u8>,
    pub off: i64,
    pub stack: Vec<Box<dyn PushDecoder>>,
}

impl RealDecoder {
    pub fn new(v: Vec<u8>) -> Self {
        RealDecoder {
            raw: v,
            off: 0,
            stack: Vec::new(),
        }
    }

    pub fn get_string_length(&mut self) -> DecoderResult<i64> {
        return match self.get_int16() {
            Err(err) => {
                Err(err)
            }
            Ok(length) => {
                if length < -1 {
                    return Err(ErrInvalidStringLength.into());
                } else if length > self.remaining() as i16{
                    self.off = self.raw.len() as i64;
                    return Err(ErrInsufficientData.into());
                }
                Ok(length as i64)
            }
        }
    }
}

impl PacketDecoder for RealDecoder {
    fn get_int8(&mut self) -> DecoderResult<i8> {
        if self.remaining() < 1 {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        }
        let tmp = self.raw[self.off as usize] as i8;
        self.off += 1;
        Ok(tmp)
    }

    fn get_int16(&mut self) -> DecoderResult<i16> {
        if self.remaining() < 2 {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        }

        let (_, half) = self.raw.split_at(self.off as usize);
        let tmp = BytesMut::from(half).get_i16();
        self.off += 2;
        Ok(tmp)
    }

    fn get_int32(&mut self) -> DecoderResult<i32> {
        if self.remaining() < 4 {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        }   

        let (_, half) = self.raw.split_at(self.off as usize);
        let tmp = BytesMut::from(half).get_i32();
        self.off += 4;
        Ok(tmp)
    }

    fn get_int64(&mut self) -> DecoderResult<i64> {
        if self.remaining() < 8  {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        }

        let (_, half) = self.raw.split_at(self.off as usize);
        let tmp = BytesMut::from(half).get_i64();
        self.off += 8;
        Ok(tmp)
    }

    fn get_varint(&mut self) -> DecoderResult<i64> {
        let (_, half) = self.raw.split_at(self.off as usize);
        if let Some((tmp, n)) = i64::decode_var(half) {
            if n == 0 {
                self.off = self.raw.len() as i64;
                return Err(ErrInsufficientData.into());
            } 
            if (n as i64) < 0 {
                self.off -= n as i64;
                return Err(ErrVarintOverflow.into());
            }
            self.off += n as i64;
            return Ok(tmp);
        }
        Ok(-1)
    }

    fn get_array_length(&mut self) -> DecoderResult<u32> {
        if self.remaining() < 4 {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        }
        let (_, half) = self.raw.split_at(self.off as usize);
        let tmp = BytesMut::from(half).get_u32();
        self.off += 4;
        if tmp > self.remaining() as u32 {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        } else if tmp > 2 * u16::MAX as u32 {
            return Err(ErrInvalidArrayLength.into());
        }

        Ok(tmp)
    }

    fn get_bool(&mut self) -> DecoderResult<bool> {
        return match self.get_int8() {
            Err(err) => {
                Err(err)
            }
            Ok(b) => {
                if b != 1 {
                    return Err(ErrInvalidBool.into());
                }
                return Ok(true);
            }
        }
    }

    fn get_bytes(&mut self) -> DecoderResult<Vec<u8>> {
        return match self.get_int32() {
            Err(err) => {
                Err(err)
            }
            Ok(tmp) => {
                if tmp == -1 {
                    return Ok(Vec::new());
                }
                return self.get_raw_bytes(tmp as i64)
            }
        } 
    }

    fn get_varint_bytes(&mut self) -> DecoderResult<Vec<u8>> {
        return match self.get_varint() {
            Err(err) => {
                Err(err)
            }
            Ok(tmp) => {
                if tmp == -1 {
                    return Ok(Vec::new());
                }
                return self.get_raw_bytes(tmp)
            }
        }
    }

    fn get_raw_bytes(&mut self, length: i64) -> DecoderResult<Vec<u8>> {
        todo!()
    }

    fn get_string(&mut self) -> DecoderResult<String> {
        return match self.get_string_length() {
            Err(err) => {
                Err(err)
            } 
            Ok(length) => {
                if length == -1 {
                    return Err(ErrInvalidStringLength.into())
                }
                let tmp_str = &self.raw[self.off as usize..self.off as usize+length as usize];
                let s = String::from_utf8(tmp_str.to_vec()).unwrap();
                self.off += length;
                return Ok(s);
            }
        }
    }

    fn get_nullable_string(&mut self) -> DecoderResult<Option<String>> {
        return match self.get_string_length() {
            Err(err) => {
                Err(err)
            } 
            Ok(length) => {
                if length == -1 {
                    return Err(ErrInvalidStringLength.into())
                }
                let tmp_str = &self.raw[self.off as usize..self.off as usize+length as usize];
                let s = String::from_utf8(tmp_str.to_vec()).unwrap();
                self.off += length;
                return Ok(Some(s));
            }
        }
    }

    fn get_int32_array(&mut self) -> DecoderResult<Vec<i32>> {
        if self.remaining() < 4 {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        }
        let n = self.get_int32().unwrap();
        if self.remaining() < 4 * n as i64 {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        }   

        if n == 0 {
            return Err(ErrDummy.into());
        }

        if n < 0 {
            return Err(ErrInvalidArrayLength.into());
        }
        let mut v = Vec::<i32>::new();
        for i in 0..n {
            let num = self.get_int32().unwrap();
            v.push(num);
        }
        Ok(v)
    }

    fn get_int64_array(&mut self) -> DecoderResult<Vec<i64>> {
        if self.remaining() < 4 {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        }
        let n = self.get_int32().unwrap();
        if self.remaining() < 8 * n as i64 {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        }

        if n == 0 {
            return Err(ErrDummy.into());
        }

        if n < 0 {
            return Err(ErrInvalidArrayLength.into());
        }

        let mut v = Vec::<i64>::new();
        for i in 0..n {
            let num = self.get_int64().unwrap();
            v.push(num);
        }
        Ok(v)
    }

    fn get_string_array(&mut self) -> DecoderResult<Vec<String>> {
        if self.remaining() < 4 {
            self.off = self.raw.len() as i64;
            return Err(ErrInsufficientData.into());
        }   
        let n = self.get_int32().unwrap();

        if n == 0 {
            return Err(ErrDummy.into());
        }

        if n < 0 {
            return Err(ErrInsufficientData.into());
        }

        let mut v = Vec::<String>::new();
        for i in 0..n {
            let res = self.get_string();
            if res.is_ok() {
                v.push(res.unwrap());
            } else {
                return Err(res.err().unwrap())
            }
        }        
        Ok(v)
    }

    #[inline]
    fn remaining(&mut self) -> i64 {
        (self.raw.len() - self.off as usize) as i64
    }

    fn get_subset(&mut self, length: i64) -> DecoderResult<Box<dyn PacketDecoder>> {
        return match self.get_raw_bytes(length) {
            Err(err) => {
                Err(err)
            } 
            Ok(bs) => {
                Ok(Box::new(RealDecoder::new(bs)))            
            }
        }
    }

    fn peek(&mut self, offset: i64, length: i64) -> DecoderResult<Box<dyn PacketDecoder>> {
        if self.remaining() < offset + length {
            return Err(ErrInsufficientData.into());
        }
        let off = self.off + offset;
        let start = off as usize;
        let end = off as usize + length as usize;  
        let subset = &self.raw[start..end].to_vec();
        Ok(Box::new(RealDecoder::new(subset.to_owned())))
    }

    fn peek_int8(&mut self, offset: i64) -> DecoderResult<i8> {
        let byte_len: i64 = 1;
        if self.remaining() < offset + byte_len {
            return Err(ErrInsufficientData.into());
        } 
        let v = &self.raw[self.off as usize..offset as usize][0];
        Ok(*v as i8)
    }

    fn discard(&mut self, length: i64) {
        todo!()
    }

    fn push(&mut self, push_decoder: &mut dyn PushDecoder) -> Option<Box<dyn error::Error>> {
        push_decoder.save_offset(self.off);

        let mut reserve = 0;
        todo!()
    }

    fn pop(&mut self) -> Option<Box<dyn error::Error>> {
        todo!()
    }
}