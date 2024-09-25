
use crate::{DeviceObject, 
            people::PeopleObject, 
            ExtentionObject, 
            file::FileObject, 
            Serialize, Deserialize, 
            ObjectId, ObjectTypeCode, 
            NearResult, NearError, ErrorCode, 
            PublicKey, Area, thing::ThingObject};

#[derive(Clone, Default)]
pub enum AnyNamedObject {
    #[default]
    None,
    Device(DeviceObject),
    Service(DeviceObject),
    People(PeopleObject),
    Extention(ExtentionObject),
    File(FileObject),
    Thing(ThingObject),
}

macro_rules! match_anynamed_obj {
    ($on:ident, $o:ident, $body:tt, ) => {
        match $on {
            Self::None => { panic!("fatal: object is none") }
            Self::Device($o) => $body,
            Self::Service($o) => $body,
            Self::People($o) => $body,
            Self::Extention($o) => $body,
            Self::File($o) => $body,
            Self::Thing($o) => $body,
        }
    };
}

impl AnyNamedObject {
    pub fn object_id(&self) -> &ObjectId {
        match_anynamed_obj!(self, o, { o.object_id() }, )
    }

    pub fn object_type_code(&self) -> ObjectTypeCode {
        match_anynamed_obj!(self, o, { o.desc().object_type_code() }, )
    }

    pub fn owner(&self) -> Option<&ObjectId> {
        match_anynamed_obj!(self, o, { o.desc().owner() }, )
    }

    pub fn author(&self) -> Option<&ObjectId> {
        match_anynamed_obj!(self, o, { o.desc().author() }, )
    }

    pub fn public_key(&self) -> Option<&PublicKey> {
        match_anynamed_obj!(self, o, { o.desc().public_key() }, )
    }

    pub fn area(&self) -> Option<&Area> {
        match_anynamed_obj!(self, o, { o.desc().area() }, )
    }
}

impl Serialize for AnyNamedObject {
    fn raw_capacity(&self) -> usize {
        match self {
            Self::None => { panic!("fatal: object is none") }
            Self::Device(o) => o.raw_capacity(),
            Self::Service(o) => o.raw_capacity(),
            Self::People(o) => o.raw_capacity(),
            Self::Extention(o) => o.raw_capacity(),
            Self::File(o) => o.raw_capacity(),
            Self::Thing(o) => o.raw_capacity(),
        }
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        match self {
            Self::None => { panic!("fatal: object is none") }
            Self::Device(o) => o.serialize(buf),
            Self::Service(o) => o.serialize(buf),
            Self::People(o) => o.serialize(buf),
            Self::Extention(o) => o.serialize(buf),
            Self::File(o) => o.serialize(buf),
            Self::Thing(o) => o.serialize(buf),
        }
    }
}

impl Deserialize for AnyNamedObject {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let object_type_code = {
            let (o, _) = ObjectId::deserialize(buf)?;

            o.object_type_code()?
        };

        match object_type_code {
            ObjectTypeCode::Device(_) => DeviceObject::deserialize(buf).map(| (o, buf) | (Self::Device(o), buf)),
            ObjectTypeCode::Service(_) => DeviceObject::deserialize(buf).map(| (o, buf) | (Self::Service(o), buf)),
            ObjectTypeCode::Extention => ExtentionObject::deserialize(buf).map(| (o, buf) | (Self::Extention(o), buf)),
            ObjectTypeCode::People => PeopleObject::deserialize(buf).map(| (o, buf) | (Self::People(o), buf)),
            ObjectTypeCode::File => FileObject::deserialize(buf).map(| (o, buf) | (Self::File(o), buf)),
            ObjectTypeCode::Thing => ThingObject::deserialize(buf).map(| (o, buf) | (Self::Thing(o), buf)),
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("Parsing not supported in AnyNamedObject, object_type_code={}.", object_type_code)))
            }
        }
    }
}

impl std::fmt::Display for AnyNamedObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => { write!(f, "warn: object is none") }
            Self::Device(o) => o.fmt(f),
            Self::Service(o) => o.fmt(f),
            Self::People(o) => o.fmt(f),
            Self::Extention(o) => o.fmt(f),
            Self::File(o) => o.fmt(f),
            Self::Thing(o) => o.fmt(f,)
        }
    }
}
