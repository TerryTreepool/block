
use near_base::{sequence::SequenceString, *};

use crate::package::PackageBodyTrait;

#[derive(Clone, Default)]
pub struct Ack {
    pub sequence: SequenceString,
    pub index: u8,
    pub timestamp: u64,
}

impl Serialize for Ack {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.sequence.raw_capacity() +
        self.index.raw_capacity() + 
        self.timestamp.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.sequence.serialize(buf)?;
        let buf = self.index.serialize(buf)?;
        let buf = self.timestamp.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for Ack {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }
    
        let (sequence, buf) = SequenceString::deserialize(buf)?;
        let (index, buf) = u8::deserialize(buf)?;
        let (timestamp, buf) = u64::deserialize(buf)?;

        Ok((Self{
            sequence, index, timestamp,
        }, buf))
    }

}

impl std::fmt::Display for Ack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ack: version: {}, sequence: {}, index: {}, timestamp: {}", Self::version(), self.sequence, self.index, self.timestamp)
    }
}

impl PackageBodyTrait for Ack {
    fn version() -> u8 {
        1u8
    }
}
