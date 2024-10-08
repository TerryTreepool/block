
use near_base::{NamedObjectDesc, NamedObjectBody, NamedObject, Serialize, Deserialize, ObjectBuilder, NearResult, ObjectBodyTrait, ObjectTypeCode, ObjectDescTrait, ObjectId, Area, PublicKey, NearError, ErrorCode};
use near_transport::ItfTrait;

pub type RawObjectDesc = NamedObjectDesc<RawObjectDescContent>;
pub type RawObjectBody = NamedObjectBody<RawObjectBodyContent>;
pub type RawObjectBuilder = ObjectBuilder<RawObjectDescContent, RawObjectBodyContent>;
pub type RawObject = NamedObject<RawObjectDescContent, RawObjectBodyContent>;

const DATACONTENT_NONE_DEFAULT: u8      = 0u8;
const DATACONTENT_ERROR_DEFAULT: u8     = 1u8;
const DATACONTENT_CONTENT_DEFAULT: u8   = 2u8;

#[derive(Clone, Default)]
pub enum RawContent {
    #[default]
    None,
    Error(NearError),
    Content(RawData),
}

#[derive(Clone, Default)]
pub struct RawData {
    pub format: u8,
    pub data: Vec<u8>,
}

impl std::fmt::Display for RawContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Error(e) => write!(f, "error={e}"),
            Self::Content(c) => write!(f, "content: format={}, len={}", c.format, c.data.len())
        }
    }
}

impl Serialize for RawContent {
    fn raw_capacity(&self) -> usize {
        match self {
            Self::None => DATACONTENT_NONE_DEFAULT.raw_capacity(),
            Self::Error(e) => DATACONTENT_ERROR_DEFAULT.raw_capacity() + e.raw_capacity(),
            Self::Content(c) => DATACONTENT_CONTENT_DEFAULT.raw_capacity() + c.format.raw_capacity() + c.data.raw_capacity(),
        }
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        match self {
            Self::None => {
                DATACONTENT_NONE_DEFAULT.serialize(buf)
            }
            Self::Error(e) => {
                let buf = DATACONTENT_ERROR_DEFAULT.serialize(buf)?;
                let buf = e.serialize(buf)?;

                Ok(buf)
            }
            Self::Content(c) => {
                let buf = DATACONTENT_CONTENT_DEFAULT.serialize(buf)?;
                let buf = c.format.serialize(buf)?;
                let buf = c.data.serialize(buf)?;

                Ok(buf)
            }
        }
    }
}

impl Deserialize for RawContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (flag, buf) = u8::deserialize(buf)?;

        match flag {
            DATACONTENT_NONE_DEFAULT => {
                Ok((Self::None, buf))
            }
            DATACONTENT_ERROR_DEFAULT => {
                let (error, buf) = NearError::deserialize(buf)?;

                Ok((Self::Error(error), buf))
            }
            DATACONTENT_CONTENT_DEFAULT => {
                let (format, buf) = u8::deserialize(buf)?;
                let (data, buf) = Vec::<u8>::deserialize(buf)?;

                Ok((Self::Content(RawData { format, data }), buf))
            }
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, format!("[{flag}] can't identify RawContent flag")))
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct RawObjectDescContent {
    pub(crate) version: u8,
    pub(crate) data: RawContent,
}

impl RawObjectDescContent {

    pub fn set_version(mut self, version: u8) -> Self {
        self.version = version;
        self
    }

    pub fn set_with_data(mut self, format: u8, data: Vec<u8>) -> Self {
        self.data = RawContent::Content(RawData{format, data});
        self
    }

    pub fn set_with_error(mut self, error: NearError) -> Self {
        self.data = RawContent::Error(error);
        self
    }

    #[inline]
    pub fn version(&self) -> u8 {
        self.version
    }

    #[inline]
    pub fn data(&self) -> &RawContent {
        &self.data
    }

}

impl std::fmt::Display for RawObjectDescContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "raw-desc: {{version: {}, data: {}}}", self.version(), self.data)
    }
}

impl Serialize for RawObjectDescContent {
    fn raw_capacity(&self) -> usize {
        self.version.raw_capacity() + 
        self.data.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        let buf = self.version.serialize(buf)?;
        let buf = self.data.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for RawObjectDescContent {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        let (version, buf) = u8::deserialize(buf)?;
        let (data, buf) = RawContent::deserialize(buf)?;

        Ok((Self{
            version, data
        }, buf))
    }
}

#[derive(Clone)]
pub struct RawObjectGuard(RawObject);

impl std::ops::Deref for RawObjectGuard {
    type Target = RawObject;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ItfTrait for RawObjectGuard {}

impl Serialize for RawObjectGuard {
    fn raw_capacity(&self) -> usize {
        self.0.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.0.serialize(buf)
    }
}

impl Deserialize for RawObjectGuard {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (r, buf) = RawObject::deserialize(buf)?;

        Ok((Self(r), buf))
    }
}

impl From<RawObject> for RawObjectGuard {
    fn from(value: RawObject) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for RawObjectGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// impl ItfTrait for NamedObject<RawObjectDescContent, RawObjectBodyContent> {}

// impl Serialize for RawObjectDescContent {
//     fn raw_capacity(&self) -> usize {
//         self.id.raw_capacity() +
//         (self.type_code as u8).raw_capacity() +
//         self.begin_time.raw_capacity()
//     }

//     fn serialize<'a>(&self,
//                      buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
//         let buf = self.id.serialize(buf)?;
//         let buf = (self.type_code as u8).serialize(buf)?;
//         let buf = self.begin_time.serialize(buf)?;

//         Ok(buf)
//     }
// }

// impl Deserialize for RawObjectDescContent {
//     fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
//         let (id, buf) = u32::deserialize(buf)?;
//         let (type_code, buf) = u8::deserialize(buf)?;
//         let (begin_time, buf) = Timestamp::deserialize(buf)?;

//         Ok((Self{
//             id, 
//             type_code: RawObjectTypeCode::try_from(type_code)?,
//             begin_time,
//         },
//         buf))
//     }
    
// }

impl ObjectDescTrait for RawObjectDescContent {
    fn object_type_code(&self) -> ObjectTypeCode {
        ObjectTypeCode::with_raw()
    }

    type OwnerObj = ObjectId;
    type AreaObj = Area;
    type AuthorObj = ObjectId;
    type PublicKeyObj = PublicKey;

}

#[derive(Clone, Default)]
pub struct RawObjectBodyContent {  }

impl Serialize for RawObjectBodyContent {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        Ok(buf)
    }
}

impl Deserialize for RawObjectBodyContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        Ok((Self{},
        buf))
    }
}

impl ObjectBodyTrait for RawObjectBodyContent { }

impl std::fmt::Display for RawObjectBodyContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "raw-body: {{None}}")
    }
}

mod test {

    #[test]
    fn test_raw_str() {
        // use super::RawObjectDescContent;
        // use crate::raw_object::RawObjectBodyContent;

        // let desc = RawObjectDescContent {
        //     version: 1,
        //     format: 1u8,
        //     data: vec![1,2,3,4,5,6],
        // };
        // let obj = 
        //     super::RawObjectBuilder::new(desc, RawObjectBodyContent {})
        //         .build()
        //         .unwrap();

        // println!("{}", obj);
        // super::RawObjectBuilder::new()
    }
}
