
use crate::{codec::{Serialize, Deserialize},
            components::{ObjectDescTrait, ObjectBodyTrait, ObjectTypeCode, Area},
            errors::*,
            public_key::PublicKey};

use super::{object_impl::*, object_type::*};

pub type ThingDesc = NamedObjectDesc<ThingDescContent>;
pub type ThingBody = NamedObjectBody<ThingBodyContent>;
pub type ThingObject = NamedObject<ThingDescContent, ThingBodyContent>;

#[derive(Clone)]
pub struct ThingDescContent {
    object_type_code: ObjectTypeCode,
    id: ObjectId,
}

impl ObjectDescTrait for ThingDescContent {
    fn object_type_code(&self) -> ObjectTypeCode {
        self.object_type_code
    }

    type OwnerObj = ObjectId;
    type AreaObj = Area;
    type AuthorObj = ObjectId;
    type PublicKeyObj = PublicKey;
}

impl ThingDescContent {
    pub fn new(code: u8) -> Self {
        Self { 
            object_type_code: ObjectTypeCode::Thing(code), 
            id: ObjectId::default()
        }
    }
}

impl Serialize for ThingDescContent {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.object_type_code.serialize(buf)?;

        let buf = self.id.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for ThingDescContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (object_type_code, buf) = ObjectTypeCode::deserialize(buf)?;

        let (id, buf) = ObjectId::deserialize(buf)?;

        Ok((Self{
            object_type_code, id
        }, buf))
    }

}

#[derive(Clone)]
pub struct ThingBodyContent {
    // endpoints, sn_list, pn_list, name, owner
    // owner: Option<>
    // endpoints, sn_list, pn_list, name, owner

}

impl ObjectBodyTrait for ThingBodyContent {}

// pub struct Thing {
// }
impl Serialize for ThingBodyContent {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        Ok(buf)
    }

}

impl Deserialize for ThingBodyContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        Ok((Self{}, buf))
    }

}
