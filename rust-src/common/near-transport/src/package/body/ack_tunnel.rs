use near_base::*;

use crate::package::PackageBodyTrait;

#[derive(Clone, Default)]
pub struct AckTunnel {
    // pub sequence: SequenceString,
    pub result: u16,
    pub send_time: u64,
}

impl Serialize for AckTunnel {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        // self.sequence.raw_capacity() +
        self.result.raw_capacity() + 
        self.send_time.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.result.serialize(buf)?;
        let buf = self.send_time.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for AckTunnel {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }
    
        let (result, buf) = u16::deserialize(buf)?;
        let (send_time, buf) = u64::deserialize(buf)?;


        Ok((Self { result, send_time }, buf))

    }
}

impl std::fmt::Display for AckTunnel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AckTunnel: version: {}, result: {}, send_time: {}",
            Self::version(), 
            self.result, self.send_time
        )
    }
}

impl PackageBodyTrait for AckTunnel {
    fn version() -> u8 {
        1u8
    }
}
