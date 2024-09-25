
use crate::errors::{NearResult, NearError, ErrorCode};

pub struct BuilderCounter {
    counter: u8,
}

impl BuilderCounter {
    pub fn new() -> Self {
        Self { counter: 0u8 }
    }

    pub fn next(&mut self) -> u8 {
        self.counter += 1;
        self.counter
    }

    pub fn curr(&self) -> u8 {
        self.counter
    }
}

pub const SERIALIZE_HEADER_SIZE: usize = 2;

pub trait Serialize {
    fn raw_capacity(&self) -> usize;
    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]>;

    fn serialize_head<'a>(&self,
                          buf: &'a mut [u8],
                          builder: &mut BuilderCounter) -> NearResult<(&'a mut [u8] /* remain buf */, usize /* capacity */)> {
        let size = self.raw_capacity();

        if size > std::mem::size_of::<u8>() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "size too long"));
        }
        if buf.len() < size + SERIALIZE_HEADER_SIZE {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
        }

        let cur = buf;
        let cur = { cur[0] = builder.next(); &mut cur[1..] };
        let cur = { cur[0] = size as u8; &mut cur[1..] };

        Ok((cur, size))
    }


}

pub trait Deserialize: Sized {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])>;
}
