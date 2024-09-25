
use near_base::{*, any::AnyNamedObject};

use crate::package::PackageBodyTrait;

#[derive(Clone, Default)]
pub struct Exchange {
    // pub sequence: TempSeq,
    // pub seq_key_sign: Signature,
    // pub from_device_id: DeviceId,
    pub aes_key: AesKey,
    pub send_time: Timestamp,
    pub from_device: AnyNamedObject,
}

impl Serialize for Exchange {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.send_time.raw_capacity() +
        self.aes_key.raw_capacity() + 
        self.from_device.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.send_time.serialize(buf)?;
        let buf = self.aes_key.serialize(buf)?;
        let buf = self.from_device.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for Exchange {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }
    
        let (send_time, buf) = Timestamp::deserialize(buf)?;
        let (aes_key, buf) = AesKey::deserialize(buf)?;
        let (from_device, buf) = AnyNamedObject::deserialize(buf)?;

        Ok((Self{
            send_time, aes_key, from_device,
        }, buf))
    }

}

impl std::fmt::Display for Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Exchange: version: {}, aes_key: {{****}}, send_time: {}, from_device: {}", Self::version(), self.send_time, self.from_device)
    }
}

impl PackageBodyTrait for Exchange {
    fn version() -> u8 {
        1u8
    }
}

