
use crate::{components::{ObjectTypeCode}, 
            Serialize, Deserialize,
            public_key::PublicKey,
            errors::*, ObjectBodyTrait, ObjectDescTrait,
            Area};

use super::{object_type::{ObjectId},
            object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody},
            /* general::EndpointExt */};

pub type DeviceDesc = NamedObjectDesc<DeviceDescContent>;
pub type DeviceBody = NamedObjectBody<DeviceBodyContent>;
pub type DeviceObject = NamedObject<DeviceDescContent, DeviceBodyContent>;

#[derive(Clone)]
pub struct DeviceDescContent {
    object_type_code: ObjectTypeCode,

    id: ObjectId
}

impl DeviceDescContent {
    pub fn new(code: u8) -> Self {
        Self {
            object_type_code: ObjectTypeCode::Device(code), 
            id: ObjectId::default()
        }
    }
}

impl ObjectDescTrait for DeviceDescContent {
    fn object_type_code(&self) -> ObjectTypeCode {
        self.object_type_code
    }

    type OwnerObj = ObjectId;
    type AreaObj = Area;
    type AuthorObj = ObjectId;
    type PublicKeyObj = PublicKey;

}

impl Serialize for DeviceDescContent {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.object_type_code.serialize(buf)?;

        let buf = self.id.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for DeviceDescContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (object_type_code, buf) = ObjectTypeCode::deserialize(buf)?;

        let (id, buf) = ObjectId::deserialize(buf)?;

        Ok((Self{
            object_type_code, id
        }, buf))
    }

}

#[derive(Clone)]
pub struct DeviceBodyContent {
    // endpoint_ext: EndpointExt,
    name: Option<String>,
}

impl ObjectBodyTrait for DeviceBodyContent { 
}

impl Serialize for DeviceBodyContent {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        // let buf = self.endpoint_ext.serialize(buf)?;

        let buf = self.name.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for DeviceBodyContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        // let (endpoint_ext, buf) = EndpointExt::deserialize(buf)?;

        let (name, buf) = Option::<String>::deserialize(buf)?;

        Ok((Self{
            /* endpoint_ext,  */name
        }, buf))
    }

}
