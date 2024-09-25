
use near_base::DeviceObject;

use super::BodyTrait;

#[derive(Default)]
pub struct Search {
}

impl near_base::Serialize for Search {
    fn raw_capacity(&self) -> usize {
        0usize
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        Ok(buf)
    }
}

impl near_base::Deserialize for Search {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        Ok((Self {
        }, buf))
    }
}

impl BodyTrait for Search {}

#[derive(Default)]
pub struct SearchResp {
    pub desc: DeviceObject,
}

impl near_base::Serialize for SearchResp {
    fn raw_capacity(&self) -> usize {
        self.desc.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        let buf = self.desc.serialize(buf)?;

        Ok(buf)
    }
}

impl near_base::Deserialize for SearchResp {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        let (desc, buf) = DeviceObject::deserialize(buf)?;

        Ok((Self{
            desc,
        }, buf))
    }
}

impl BodyTrait for SearchResp {}
