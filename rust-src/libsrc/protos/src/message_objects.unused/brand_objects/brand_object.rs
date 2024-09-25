
use near_base::{Timestamp, NamedObjectDesc, NamedObjectBody, NamedObject, ObjectDescTrait, ObjectTypeCode, ObjectId, Area, PublicKey, ObjectBuilder, now, NearResult, Serialize, Deserialize, ObjectBodyTrait, NearError, ErrorCode};
use near_util::BrandOptionTypeCode;

use crate::brand::Brand_info;

#[repr(u8)]
#[derive(Clone, Copy, Default)]
pub enum BrandStatus {
    #[default]
    Enable = 0,
    Disable,
}

impl TryFrom<u8> for BrandStatus {
    type Error = NearError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0u8    => Ok(Self::Enable),
            1u8    => Ok(Self::Disable),
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, 
                                   format!("{value} is invalid value in brand status.")))
            }
        }
    }
}

impl std::fmt::Display for BrandStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match &self {
            Self::Enable => "enable",
            Self::Disable => "disable",
        };

        write!(f, "{status}")
    }
}

pub type BrandOptionDesc = NamedObjectDesc<BrandOptionDescContent>;
pub type BrandOptionBody = NamedObjectBody<BrandOptionBodyContent>;
pub type BrandOptionObject = NamedObject<BrandOptionDescContent, BrandOptionBodyContent>;

#[derive(Clone)]
pub struct BrandOptionDescContent {
    pub(crate) id: u32,
    pub(crate) type_code: BrandOptionTypeCode,
    pub(crate) begin_time: Timestamp,
}

impl BrandOptionDescContent {
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn type_code(&self) -> BrandOptionTypeCode {
        self.type_code
    }

    #[inline]
    pub fn begin_time(&self) -> Timestamp {
        self.begin_time
    }
}

impl std::fmt::Display for BrandOptionDescContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Brand-desc: {{id: {}, object_type_code: {}, begin_time: {}}}", self.id(), self.object_type_code(), self.begin_time())
    }
}

impl Serialize for BrandOptionDescContent {
    fn raw_capacity(&self) -> usize {
        self.id.raw_capacity() +
        (self.type_code as u8).raw_capacity() +
        self.begin_time.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.id.serialize(buf)?;
        let buf = (self.type_code as u8).serialize(buf)?;
        let buf = self.begin_time.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for BrandOptionDescContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (id, buf) = u32::deserialize(buf)?;
        let (type_code, buf) = u8::deserialize(buf)?;
        let (begin_time, buf) = Timestamp::deserialize(buf)?;

        Ok((Self{
            id, 
            type_code: BrandOptionTypeCode::try_from(type_code)?,
            begin_time,
        },
        buf))
    }
    
}

impl ObjectDescTrait for BrandOptionDescContent {
    fn object_type_code(&self) -> ObjectTypeCode {
        ObjectTypeCode::with_brand_option(self.type_code.into())
    }

    type OwnerObj = ObjectId;
    type AreaObj = Area;
    type AuthorObj = ObjectId;
    type PublicKeyObj = PublicKey;

}

#[derive(Clone)]
pub struct BrandOptionBodyContent {
    pub(crate) name: String,
    pub(crate) status: BrandStatus,
    pub(crate) update_time: Timestamp,
}

impl BrandOptionBodyContent {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn status(&self) -> BrandStatus {
        self.status
    }

    #[inline]
    pub fn update_time(&self) -> Timestamp {
        self.update_time
    }

}

impl Serialize for BrandOptionBodyContent {
    fn raw_capacity(&self) -> usize {
        self.name.raw_capacity() +
        (self.status as u8).raw_capacity() +
        self.update_time.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.name.serialize(buf)?;
        let buf = (self.status as u8).serialize(buf)?;
        let buf = self.update_time.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for BrandOptionBodyContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (name, buf) = String::deserialize(buf)?;
        let (status, buf) = u8::deserialize(buf)?;
        let (update_time, buf) = Timestamp::deserialize(buf)?;

        Ok((Self{
            name, 
            status: BrandStatus::try_from(status)?,
            update_time,
        },
        buf))
    }
}

impl ObjectBodyTrait for BrandOptionBodyContent { }

impl std::fmt::Display for BrandOptionBodyContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Brand-Body: {{name: {}, status: {}, update_time={}}}", self.name(), self.status(), self.update_time())
    }
}

#[derive(Default)]
pub struct BrandOptionBuilder {
    id: u32,
    status: BrandStatus,
    type_code: BrandOptionTypeCode,
    name: String,
}

impl BrandOptionBuilder {
    pub fn set_id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    pub fn set_status(mut self, status: BrandStatus) -> Self {
        self.status = status;
        self
    }

    pub fn set_type_code(mut self, type_code: BrandOptionTypeCode) -> Self {
        self.type_code = type_code;
        self
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }
}

impl BrandOptionBuilder {
    pub fn build(self) -> NearResult<BrandOptionObject> {

        debug_assert!(self.type_code as u8 != BrandOptionTypeCode::default() as u8);

        let now = now();

        ObjectBuilder::new(BrandOptionDescContent {
                                id: self.id,
                                type_code: self.type_code,
                                begin_time: now,
                            },
                            BrandOptionBodyContent {
                            name: self.name,
                            status: self.status,
                            update_time: now,
                            })
            .build()
    }
}

impl From<&BrandOptionObject> for Brand_info {

    fn from(value: &BrandOptionObject) -> Self {
        
        let mut brand = Brand_info::default();

        // let fmt = "%Y-%m-%d %H:%M:%S";
        // let date_str = std::time::Duration::from_micros(micros)

        // brand.set_brand_id(value.desc().content().id());
        // brand.set_brand_name(value.body().content().name().to_owned());
        // brand.set_begin_time(v)

        brand
    }
}

mod test {
    #[test]
    fn test_date_str() {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let now = near_base::now();
        let now = std::time::Duration::from_millis(now);
        println!("{:#?}", now);
    }
}
