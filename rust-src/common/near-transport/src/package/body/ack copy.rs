
use near_base::*;

use crate::package::PackageBodyTrait;

#[derive(Clone, Default)]
pub struct Ack {
    pub result: u16,
    pub send_time: u64,
}

impl Serialize for Ack {
    fn raw_capacity(&self) -> usize {
        self.result.raw_capacity() + 
        self.send_time.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.result.serialize(buf)?;
        let buf = self.send_time.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for Ack {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (result, buf) = u16::deserialize(buf)?;
        let (send_time, buf) = u64::deserialize(buf)?;

        Ok((Self{
            result, send_time,
        }, buf))
    }

}

impl std::fmt::Display for Ack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ack: result: {}, send_time: {}", self.result, self.send_time)
    }
}

impl PackageBodyTrait for Ack {}
