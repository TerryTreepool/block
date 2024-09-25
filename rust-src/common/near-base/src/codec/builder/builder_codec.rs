
use crate::errors::*;
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
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]>;
}

pub trait SerializeWithContext {
    fn serialize_with_content<'a, Context>(&self,
                                           buf: &'a mut [u8],
                                           context: Context) -> NearResult<&'a mut [u8]>;
}

pub trait Deserialize: Sized {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])>;
}

pub trait DeserializeWithContexxt<Context>: Sized {
    fn deserialize<'de>(buf: &'de [u8], context: Context) -> NearResult<(Self, &'de [u8])>;
}

pub trait RawFixedBytes {
    fn raw_bytes() -> usize;
}
