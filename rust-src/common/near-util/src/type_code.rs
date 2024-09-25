use near_base::{NearError, ErrorCode};


#[derive(Clone, Copy)]
pub enum ServiceTypeCode {

}

#[derive(Clone, Copy)]
pub enum DeviceTypeCode {

}

#[derive(Clone, Copy)]
pub enum ExtentionTypeCode {

}

#[derive(Clone, Copy)]
pub enum PeopleTypeCode {

}

#[derive(Clone, Copy)]
pub enum FileOptionTypeCode {

}

#[repr(u8)]
#[derive(Clone, Copy, Default)]
pub enum BrandOptionTypeCode {
    #[default]
    None = 0,
    AddBrand,
    GetBrand,
    SetBrand,
}

// impl Into<u8> for BrandOptionTypeCode {
//     fn into(self) -> u8 {
//         self as u8
//     }
// }

impl From<BrandOptionTypeCode> for u8 {
    fn from(v: BrandOptionTypeCode) -> Self {
        match v {
            BrandOptionTypeCode::None => 0u8,
            BrandOptionTypeCode::AddBrand => 1u8,
            BrandOptionTypeCode::GetBrand => 2u8,
            BrandOptionTypeCode::SetBrand => 3u8,
        }
    }
}

impl TryFrom<u8> for BrandOptionTypeCode {
    type Error = NearError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0u8 => Ok(Self::None),
            1u8 => Ok(Self::AddBrand),
            2u8 => Ok(Self::GetBrand),
            3u8 => Ok(Self::SetBrand),
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, 
                                   format!("{value} is invalid value in brand option.")))
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum ThingTypeCode {

}
