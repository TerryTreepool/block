
use crate::errors::*;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum DeviceType {
    Unknown = 0,

    // system server
    Service = 1,

    // android
    AndroidMobile = 51,
    AndroidPad = 52,
    AndroidWatch = 53,
    AndroidTV = 54,

    // iOS
    IOSMobile = 61,
    IOSPad = 62,
    IOSWatch = 63,

    // smart & ioT
    SmartSpeakers = 71,

    // other
    Browser = 101,
}

impl TryFrom<u8> for DeviceType {
    type Error = ErrorCode;
    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        match v {
            1u8 => Ok(DeviceType::Service),
            51u8 => Ok(DeviceType::AndroidMobile),
            52u8 => Ok(DeviceType::AndroidPad),
            53u8 => Ok(DeviceType::AndroidWatch),
            54u8 => Ok(DeviceType::AndroidTV),
            61u8 => Ok(DeviceType::IOSMobile),
            62u8 => Ok(DeviceType::IOSPad),
            63u8 => Ok(DeviceType::IOSWatch),
            71u8 => Ok(DeviceType::SmartSpeakers),
            101u8 => Ok(DeviceType::Browser),
            0u8 | _ => Ok(DeviceType::Unknown),
        }
    }
}

impl Into<u8> for DeviceType {
    fn into(self) -> u8 {
        match self {
            DeviceType::Service => 1u8,
            DeviceType::AndroidMobile => 51u8,
            DeviceType::AndroidPad => 52u8,
            DeviceType::AndroidWatch => 53u8,
            DeviceType::AndroidTV => 54u8,
            DeviceType::IOSMobile => 61u8,
            DeviceType::IOSPad => 62u8,
            DeviceType::IOSWatch => 63u8,
            DeviceType::SmartSpeakers => 71u8,
            DeviceType::Browser => 101u8,
            DeviceType::Unknown => 0u8,
        }
    }
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DeviceType::Service => write!(f, "(1)Service"),
            DeviceType::AndroidMobile => write!(f, "(51)AndroidMobile"),
            DeviceType::AndroidPad => write!(f, "(52)AndroidPad"),
            DeviceType::AndroidWatch => write!(f, "(53)AndroidWatch"),
            DeviceType::AndroidTV => write!(f, "(54)AndroidTV"),
            DeviceType::IOSMobile => write!(f, "(61)IOSMobile"),
            DeviceType::IOSPad => write!(f, "(62)IOSPad"),
            DeviceType::IOSWatch => write!(f, "(63)IOSWatch"),
            DeviceType::SmartSpeakers => write!(f, "(71)SmartSpeakers"),
            DeviceType::Browser => write!(f, "(101)Browser"),
            _ => write!(f, "(0)unknown"),
        }
    }
    
}
