
use near_base::{sequence::SequenceString, *};

use crate::package::PackageBodyTrait;

#[derive(Clone, Default)]
pub struct AckAck {
    pub sequence: SequenceString,
    pub index: u8,
    pub errno: u16, // ErrorCode
}

impl Serialize for AckAck {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.sequence.raw_capacity() +
        self.index.raw_capacity() + 
        self.errno.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.sequence.serialize(buf)?;
        let buf = self.index.serialize(buf)?;
        let buf = self.errno.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for AckAck {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }
    
        let (sequence, buf) = SequenceString::deserialize(buf)?;
        let (index, buf) = u8::deserialize(buf)?;
        let (errno, buf) = u16::deserialize(buf)?;

        Ok((Self{
            sequence, index, errno
        }, buf))
    }

}

impl std::fmt::Display for AckAck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AckAck: version: {}, sequence: {}, index: {}, errno: {}", Self::version(), self.sequence, self.index, self.errno)
   
    }
}

impl PackageBodyTrait for AckAck {
    fn version() -> u8 {
        1u8
    }
}
