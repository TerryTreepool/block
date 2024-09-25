
use near_base::{sequence::SequenceString, *};

use crate::package::PackageBodyTrait;

#[derive(Clone, Default)]
pub struct AckAckTunnel {
    pub sequence: SequenceString,
    pub result: u16,
    pub send_time: u64,
}

impl Serialize for AckAckTunnel {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.sequence.raw_capacity() +
        self.result.raw_capacity() +
        self.send_time.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.sequence.serialize(buf)?;
        let buf = self.result.serialize(buf)?;
        let buf = self.send_time.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for AckAckTunnel {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }
    
        let (sequence, buf) = SequenceString::deserialize(buf)?;
        let (result, buf) = u16::deserialize(buf)?;
        let (send_time, buf) = u64::deserialize(buf)?;

        Ok((Self{
            sequence, result, send_time
        }, buf))

    }

}

impl std::fmt::Display for AckAckTunnel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AckAckTunnel: version: {} sequence: {}, result: {}, send_time: {}", Self::version(), self.sequence, self.result, self.send_time)
   
    }
}

impl PackageBodyTrait for AckAckTunnel {
    fn version() -> u8 {
        1u8
    }
}
