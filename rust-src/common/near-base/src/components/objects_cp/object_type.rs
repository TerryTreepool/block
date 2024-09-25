

use base58::ToBase58;

use generic_array::typenum::Unsigned;
use generic_array::{GenericArray, typenum::U32};

use crate::{Serialize, Deserialize};
use crate::errors::*;

const OBJECT_TYPE_SERVICE_CODE: u8 = 1;
const OBJECT_TYPE_DEVICE_CODE: u8 = 2;
const OBJECT_TYPE_PEOPLE_CODE: u8 = 3;
const OBJECT_TYPE_THING_CODE: u8 = 4;
const OBJECT_TYPE_OTHER_CODE: u8 = 16;

#[derive(Clone, Copy)]
pub enum ObjectTypeCode {
    Unknown,
    /// 服务属性，如sn，pn服务等
    Service(u8),
    /// 设备属性，如黑盒
    Device(u8),
    /// 用户属性，如玩家，玩家组等
    People(u8),
    /// Thing属性，如灯等
    Thing(u8),
    /// 其他属性
    Other(u8),
}

impl ObjectTypeCode {
    pub fn with_service(v: u8) -> Self {
        ObjectTypeCode::Service(v)
    }

    pub fn with_device(v: u8) -> Self {
        ObjectTypeCode::Device(v)
    }

    pub fn with_people(v: u8) -> Self {
        ObjectTypeCode::People(v)
    }

    pub fn with_thing(v: u8) -> Self {
        ObjectTypeCode::Thing(v)
    }

    pub fn with_other(v: u8) -> Self {
        ObjectTypeCode::Other(v)
    }
}

impl From<u16> for ObjectTypeCode {
    fn from(v: u16) -> Self {
        let h = (v >> 8) as u8;
        let l = (((v << 8) as u16) >> 8) as u8;

        match h {
            OBJECT_TYPE_SERVICE_CODE => ObjectTypeCode::Service(l),
            OBJECT_TYPE_DEVICE_CODE => ObjectTypeCode::Device(l),
            OBJECT_TYPE_PEOPLE_CODE => ObjectTypeCode::People(l),
            OBJECT_TYPE_THING_CODE => ObjectTypeCode::Thing(l),
            OBJECT_TYPE_OTHER_CODE => ObjectTypeCode::Other(l),
            _ => ObjectTypeCode::Unknown,
        }
    }
}

impl From<&ObjectTypeCode> for u16 {
    fn from(v: &ObjectTypeCode) -> u16 {
        let (h, l) = {
            match *v {
                ObjectTypeCode::Unknown => (0u8, 0u8),
                ObjectTypeCode::Service(v) => (OBJECT_TYPE_SERVICE_CODE, v),
                ObjectTypeCode::Device(v) => (OBJECT_TYPE_DEVICE_CODE, v),
                ObjectTypeCode::People(v) => (OBJECT_TYPE_PEOPLE_CODE, v),
                ObjectTypeCode::Thing(v) => (OBJECT_TYPE_THING_CODE, v),
                ObjectTypeCode::Other(v) => (OBJECT_TYPE_OTHER_CODE, v),
            }
        };

        let v = ((h as u16) << 8) | l as u16;
        v
    }
}

impl Serialize for ObjectTypeCode {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let v = u16::from(self);
        v.serialize(buf)
    }

}

impl Deserialize for ObjectTypeCode {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u16::deserialize(buf)?;
        Ok((ObjectTypeCode::from(v), buf))
    }

}

/// object id
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId(GenericArray<u8, U32>);

impl std::string::ToString for ObjectId {
    fn to_string(&self) -> String {
        self.0.as_slice().to_base58()
    }
}

impl ObjectId {
    pub fn capacity() -> usize {
        U32::to_usize()
    }
}

impl AsRef<GenericArray<u8, U32>> for ObjectId {
    fn as_ref(&self) -> &GenericArray<u8, U32> {
        &self.0
    }
}

impl std::default::Default for ObjectId {
    fn default() -> Self {
        Self(GenericArray::default())
    }
    
}

impl From<&GenericArray<u8, U32>> for ObjectId {
    fn from(data: &GenericArray<u8, U32>) -> Self {
        Self(data.clone())
    }
}

impl Serialize for ObjectId {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.0.as_slice().serialize(buf)
    }

}

impl Deserialize for ObjectId {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (val, buf) = Vec::<u8>::deserialize(buf)?;

        Ok((Self(GenericArray::clone_from_slice(val.as_slice())), buf))
    }

}