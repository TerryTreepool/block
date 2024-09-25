

use std::fmt::Write;
use std::str::FromStr;

use base58::{FromBase58, ToBase58, FromBase58Error};

use generic_array::typenum::Unsigned;
use generic_array::{GenericArray, typenum::U32};

use crate::{Serialize, Deserialize, };
use crate::errors::*;

#[repr(u16)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub(super) enum ObjectTypeMajorCode {
    OBJECT_TYPE_CLUSTER_CODE    = 0,
    OBJECT_TYPE_SERVICE_CODE    = 1,
    OBJECT_TYPE_DEVICE_CODE     = 2,
    OBJECT_TYPE_EXTENTION_CODE  = 3,
    OBJECT_TYPE_THING_CODE      = 4,
    OBJECT_TYPE_PEOPLE_CODE     = 6,
    OBJECT_TYPE_FILE_CODE       = 0xa,
    OBJECT_TYPE_RAW_CODE        = 0xe,
    OBJECT_TYPE_OTHER_CODE      = 0xf,
}

impl std::fmt::Display for ObjectTypeMajorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = 
            match self {
                Self::OBJECT_TYPE_CLUSTER_CODE => "Cluster",
                Self::OBJECT_TYPE_SERVICE_CODE => "Service",
                Self::OBJECT_TYPE_DEVICE_CODE => "Device",
                Self::OBJECT_TYPE_EXTENTION_CODE => "Extention",
                Self::OBJECT_TYPE_THING_CODE => "Thing",
                Self::OBJECT_TYPE_PEOPLE_CODE => "People",
                Self::OBJECT_TYPE_FILE_CODE => "File",
                Self::OBJECT_TYPE_RAW_CODE => "Raw",
                Self::OBJECT_TYPE_OTHER_CODE => "Other",
            };

        write!(f, "{v}")
    }
}

impl TryFrom<u8> for ObjectTypeMajorCode {
    type Error = NearError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::OBJECT_TYPE_CLUSTER_CODE),
            1 => Ok(Self::OBJECT_TYPE_SERVICE_CODE),
            2 => Ok(Self::OBJECT_TYPE_DEVICE_CODE),
            3 => Ok(Self::OBJECT_TYPE_EXTENTION_CODE),
            4 => Ok(Self::OBJECT_TYPE_THING_CODE),
            6 => Ok(Self::OBJECT_TYPE_PEOPLE_CODE),
            0xa => Ok(Self::OBJECT_TYPE_FILE_CODE),
            0xe => Ok(Self::OBJECT_TYPE_RAW_CODE),
            0xf => Ok(Self::OBJECT_TYPE_OTHER_CODE),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("{value} undefined.")))
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum DeviceObjectSubCode {
    OBJECT_TYPE_DEVICE_CORE     = 1,
}

impl std::fmt::Display for DeviceObjectSubCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = 
            match self {
                Self::OBJECT_TYPE_DEVICE_CORE => "core-service",
            };

        write!(f, "{v}")
    }
}

impl FromStr for DeviceObjectSubCode {
    type Err = NearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "core-service" => Ok(Self::OBJECT_TYPE_DEVICE_CORE),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("{s} undefined")))
        }        
    }
}

impl TryFrom<u8> for DeviceObjectSubCode {
    type Error = NearError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::OBJECT_TYPE_DEVICE_CORE),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("{value} undefined.")))
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum ServiceObjectSubCode {
    OBJECT_TYPE_SERVICE_COTURN_MINER    = 1,
    // OBJECT_TYPE_SERVICE_PN_MINER    = 2,

}

impl std::fmt::Display for ServiceObjectSubCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = 
            match self {
                // Self::OBJECT_TYPE_SERVICE_PN_MINER => "pn-miner",
                Self::OBJECT_TYPE_SERVICE_COTURN_MINER => "coturn-miner",
            };

        write!(f, "{v}")
    }
}

impl FromStr for ServiceObjectSubCode {
    type Err = NearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "coturn-miner" => Ok(Self::OBJECT_TYPE_SERVICE_COTURN_MINER),
            // "pn-miner" => Ok(Self::OBJECT_TYPE_SERVICE_PN_MINER),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("{s} undefined")))
        }        
    }
}

impl TryFrom<u8> for ServiceObjectSubCode {
    type Error = NearError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::OBJECT_TYPE_SERVICE_COTURN_MINER),
            // 2 => Ok(Self::OBJECT_TYPE_SERVICE_PN_MINER),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("{value} undefined.")))
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum OtherObjectSubCode {
    OBJECT_TYPE_OTHER_PROOFDATA     = 1,
}

impl std::fmt::Display for OtherObjectSubCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = 
            match self {
                Self::OBJECT_TYPE_OTHER_PROOFDATA => "proof",
            };

        write!(f, "{v}")
    }
}

impl FromStr for OtherObjectSubCode {
    type Err = NearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proof" => Ok(Self::OBJECT_TYPE_OTHER_PROOFDATA),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("{s} undefined")))
        }        
    }
}

impl TryFrom<u8> for OtherObjectSubCode {
    type Error = NearError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::OBJECT_TYPE_OTHER_PROOFDATA),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("{value} undefined.")))
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum ObjectTypeCode {
    #[default]
    Unknown,
    /// 服务属性，如sn，pn服务等
    Service(u8),
    /// 设备属性，如黑盒
    Device(u8),
    /// 插件属性
    Extention,
    /// 用户属性，如玩家，玩家组等
    People,
    /// 文件
    File,
    /// Thing属性，如灯等
    Thing,
    /// 内存数据
    Raw,
    /// 其他属性
    Other(u8),
}

impl std::fmt::Display for ObjectTypeCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, value) = match self {
            Self::Unknown => ("Unknown Object", 0u8),
            Self::Service(v) => ("Service", *v),
            Self::Device(v) => ("Device", *v),
            Self::Extention => ("Extention", 0u8),
            Self::People => ("People", 0u8),
            Self::File => ("File", 0u8),
            Self::Thing => ("Thing", 0u8),
            Self::Raw => ("Raw", 0u8),
            Self::Other(v) => ("Other", *v),
        };

        write!(f, "({name}({value}))")
    }
}

impl ObjectTypeCode {
    pub fn with_service(v: u8) -> Self {
        ObjectTypeCode::Service(v)
    }

    pub fn with_device(v: u8) -> Self {
        ObjectTypeCode::Device(v)
    }

    pub fn with_extention() -> Self {
        ObjectTypeCode::Extention
    }

    pub fn with_people() -> Self {
        ObjectTypeCode::People
    }

    pub fn with_file() -> Self {
        ObjectTypeCode::File
    }

    pub fn with_thing() -> Self {
        ObjectTypeCode::Thing
    }

    pub fn with_raw() -> Self {
        ObjectTypeCode::Raw
    }

    pub fn with_other(v: u8) -> Self {
        ObjectTypeCode::Other(v)
    }

    pub fn into_u16(&self) -> u16 {
        let (h, l) = {
            match *self {
                ObjectTypeCode::Unknown => (0u8, 0u8),
                ObjectTypeCode::Service(v) => (ObjectTypeMajorCode::OBJECT_TYPE_SERVICE_CODE as u8, v),
                ObjectTypeCode::Device(v) => (ObjectTypeMajorCode::OBJECT_TYPE_DEVICE_CODE as u8, v),
                ObjectTypeCode::Extention => (ObjectTypeMajorCode::OBJECT_TYPE_EXTENTION_CODE as u8, 0u8),
                ObjectTypeCode::People => (ObjectTypeMajorCode::OBJECT_TYPE_PEOPLE_CODE as u8, 0u8),
                ObjectTypeCode::File => (ObjectTypeMajorCode::OBJECT_TYPE_FILE_CODE as u8, 0u8),
                ObjectTypeCode::Thing => (ObjectTypeMajorCode::OBJECT_TYPE_THING_CODE as u8, 0u8),
                ObjectTypeCode::Raw => (ObjectTypeMajorCode::OBJECT_TYPE_RAW_CODE as u8, 0u8),
                ObjectTypeCode::Other(v) => (ObjectTypeMajorCode::OBJECT_TYPE_OTHER_CODE as u8, v),
            }
        };

        let v = ((h as u16) << 8) | l as u16;
        v
    }

    pub fn split(self) -> (u8 /* master */, u8 /* property */) {
        match self {
            ObjectTypeCode::Unknown => (0u8, 0u8),
            ObjectTypeCode::Service(v) => (ObjectTypeMajorCode::OBJECT_TYPE_SERVICE_CODE as u8, v),
            ObjectTypeCode::Device(v) => (ObjectTypeMajorCode::OBJECT_TYPE_DEVICE_CODE as u8, v),
            ObjectTypeCode::Extention => (ObjectTypeMajorCode::OBJECT_TYPE_EXTENTION_CODE as u8, 0u8),
            ObjectTypeCode::People => (ObjectTypeMajorCode::OBJECT_TYPE_PEOPLE_CODE as u8, 0u8),
            ObjectTypeCode::File => (ObjectTypeMajorCode::OBJECT_TYPE_FILE_CODE as u8, 0u8),
            ObjectTypeCode::Thing => (ObjectTypeMajorCode::OBJECT_TYPE_THING_CODE as u8, 0u8),
            ObjectTypeCode::Raw => (ObjectTypeMajorCode::OBJECT_TYPE_RAW_CODE as u8, 0u8),
            ObjectTypeCode::Other(v) => (ObjectTypeMajorCode::OBJECT_TYPE_OTHER_CODE as u8, v),
        }
    }

    pub fn to_string(&self) -> NearResult<String> {
        let mut text = String::default();

        match self {
            ObjectTypeCode::Service(v) => {
                let sub_code: ServiceObjectSubCode = (*v).try_into()?;
                text.write_fmt(format_args!("{}({})", ObjectTypeMajorCode::OBJECT_TYPE_SERVICE_CODE, sub_code))
                    .map_err(| e | NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("{e}")))?;
                Ok(())
            }
            ObjectTypeCode::Device(v) => {
                let sub_code: DeviceObjectSubCode = (*v).try_into()?;
                text.write_fmt(format_args!("{}({})", ObjectTypeMajorCode::OBJECT_TYPE_DEVICE_CODE, sub_code))
                    .map_err(| e | NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("{e}")))?;
                Ok(())
            }
            ObjectTypeCode::Extention => {
                text.write_fmt(format_args!("{}", ObjectTypeMajorCode::OBJECT_TYPE_EXTENTION_CODE))
                    .map_err(| e | NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("{e}")))?;
                Ok(())
            },
            ObjectTypeCode::People => {
                text.write_fmt(format_args!("{}", ObjectTypeMajorCode::OBJECT_TYPE_PEOPLE_CODE))
                    .map_err(| e | NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("{e}")))?;
                Ok(())
            }
            ObjectTypeCode::File => {
                text.write_fmt(format_args!("{}", ObjectTypeMajorCode::OBJECT_TYPE_FILE_CODE))
                    .map_err(| e | NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("{e}")))?;
                Ok(())
            }
            ObjectTypeCode::Thing => {
                text.write_fmt(format_args!("{}", ObjectTypeMajorCode::OBJECT_TYPE_THING_CODE))
                    .map_err(| e | NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("{e}")))?;
                Ok(())
            }
            ObjectTypeCode::Raw => {
                text.write_fmt(format_args!("{}", ObjectTypeMajorCode::OBJECT_TYPE_RAW_CODE))
                    .map_err(| e | NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("{e}")))?;
                Ok(())
            }
            ObjectTypeCode::Other(v) => {
                let sub_code: OtherObjectSubCode = (*v).try_into()?;
                text.write_fmt(format_args!("{}({})", ObjectTypeMajorCode::OBJECT_TYPE_OTHER_CODE, sub_code))
                    .map_err(| e | NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("{e}")))?;
                Ok(())
            }
            ObjectTypeCode::Unknown => 
                Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, "undefined"))
        }?;

        Ok(text)
    }
}

impl From<u16> for ObjectTypeCode {
    fn from(v: u16) -> Self {
        let h = (v >> 8) as u8;
        let l = (((v << 8) as u16) >> 8) as u8;

        if h == ObjectTypeMajorCode::OBJECT_TYPE_SERVICE_CODE as u8 {
            ObjectTypeCode::Service(l)
        } else if h == ObjectTypeMajorCode::OBJECT_TYPE_DEVICE_CODE as u8 {
            ObjectTypeCode::Device(l)
        } else if h == ObjectTypeMajorCode::OBJECT_TYPE_EXTENTION_CODE as u8 {
            ObjectTypeCode::Extention
        } else if h == ObjectTypeMajorCode::OBJECT_TYPE_PEOPLE_CODE as u8 {
            ObjectTypeCode::People
        } else if h == ObjectTypeMajorCode::OBJECT_TYPE_FILE_CODE as u8 { 
            ObjectTypeCode::File
        } else if h == ObjectTypeMajorCode::OBJECT_TYPE_THING_CODE as u8 {
            ObjectTypeCode::Thing
        } else if h == ObjectTypeMajorCode::OBJECT_TYPE_RAW_CODE as u8 {
            ObjectTypeCode::Raw
        } else if h == ObjectTypeMajorCode::OBJECT_TYPE_OTHER_CODE as u8 {
            ObjectTypeCode::Other(l)
        } else {
            ObjectTypeCode::Unknown
        }
    }
}

impl TryFrom<&ObjectId> for ObjectTypeCode {
    type Error = NearError;

    fn try_from(value: &ObjectId) -> Result<Self, Self::Error> {
        let slice = value.0.as_slice();

        let master = slice[0] >> 4 as u8;
        let val = slice[1];

        let r = 
        if master == ObjectTypeMajorCode::OBJECT_TYPE_SERVICE_CODE as u8 {
            ObjectTypeCode::Service(val)
        } else if master == ObjectTypeMajorCode::OBJECT_TYPE_DEVICE_CODE as u8 {
            ObjectTypeCode::Device(val)
        } else if master == ObjectTypeMajorCode::OBJECT_TYPE_EXTENTION_CODE as u8 {
            ObjectTypeCode::Extention
        } else if master == ObjectTypeMajorCode::OBJECT_TYPE_PEOPLE_CODE as u8 {
            ObjectTypeCode::People
        } else if master == ObjectTypeMajorCode::OBJECT_TYPE_FILE_CODE as u8 {
            ObjectTypeCode::File
        } else if master == ObjectTypeMajorCode::OBJECT_TYPE_THING_CODE as u8 {
            ObjectTypeCode::Thing
        } else if master == ObjectTypeMajorCode::OBJECT_TYPE_RAW_CODE as u8 {
            ObjectTypeCode::Raw
        } else if master == ObjectTypeMajorCode::OBJECT_TYPE_OTHER_CODE as u8 {
            ObjectTypeCode::Other(val)
        } else {
            ObjectTypeCode::Unknown
        };

        Ok(r)
    }
}

impl Serialize for ObjectTypeCode {
    fn raw_capacity(&self) -> usize {
        std::mem::size_of::<u16>()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.into_u16().serialize(buf)
    }

}

impl Deserialize for ObjectTypeCode {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u16::deserialize(buf)?;
        Ok((ObjectTypeCode::from(v), buf))
    }

}

/// object id
// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(GenericArray<u8, U32>);

impl std::fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Ord for ObjectId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd<ObjectId> for ObjectId {
    fn partial_cmp(&self, other: &ObjectId) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Debug).fmt(f)
    }
}

impl ObjectId {
    pub fn capacity() -> usize {
        U32::to_usize()
    }

    pub fn to_string(&self) -> String {
        self.0.as_slice().to_base58()
    }

    pub fn object_type_code(&self) -> NearResult<ObjectTypeCode> {
        ObjectTypeCode::try_from(self)
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

impl From<GenericArray<u8, U32>> for ObjectId {
    fn from(data: GenericArray<u8, U32>) -> Self {
        Self(data)
    }
}

impl FromStr for ObjectId {
    type Err = NearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = 
            FromBase58::from_base58(s)
                .map_err(| err | {
                    NearError::new(ErrorCode::NEAR_ERROR_3RD, {
                        match err {
                            FromBase58Error::InvalidBase58Length => "invalid length",
                            FromBase58Error::InvalidBase58Character(_, _) => "invalid character",
                        }
                    })
                })?;

        Ok(Self(GenericArray::clone_from_slice(v.as_slice())))
    }

}

impl Serialize for ObjectId {
    fn raw_capacity(&self) -> usize {
        self.0.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.0.serialize(buf)
    }

}

impl Deserialize for ObjectId {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = GenericArray::<u8, U32>::deserialize(buf)?;

        Ok((Self(v), buf))
    }

}

impl PartialEq<ObjectId> for &ObjectId {
    fn eq(&self, other: &ObjectId) -> bool {
        self.0.eq(&other.0)
    }
}
