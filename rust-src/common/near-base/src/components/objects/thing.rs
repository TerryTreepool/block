
use std::collections::HashMap;

use crate::{components::ObjectTypeCode, 
    Serialize, Deserialize, Area, PublicKey, ObjectDescTrait, NearResult, ObjectBodyTrait};

use super::{object_type::ObjectId,
    object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody},
    /* general::EndpointExt */};

pub type ThingDesc = NamedObjectDesc<ThingDescContent>;
pub type ThingBody = NamedObjectBody<ThingBodyContent>;
pub type ThingObject = NamedObject<ThingDescContent, ThingBodyContent>;

#[derive(Clone, Default)]
pub struct ThingDescContent {
    mac_address: [u8; 6],
    owner_depend_id: String,
}

impl ThingDescContent {
    pub fn new() -> Self {
        Self {
            mac_address: [0u8; 6],
            owner_depend_id: String::default(),
        }
    }

}

impl ThingDescContent {

    pub fn set_mac_address(&mut self, mac_address: [u8; 6]) {
        unsafe {
            std::ptr::copy(mac_address.as_ptr(), self.mac_address.as_mut_ptr(), 6);
        }
    }

    pub fn mac_address(&self) -> &[u8; 6] {
        &self.mac_address
    }

    pub fn set_owner_depend_id(&mut self, owner_depend_id: String) {
        self.owner_depend_id = owner_depend_id;
    }

    pub fn owner_depend_id(&self) -> &str {
        &self.owner_depend_id
    }
}

impl std::fmt::Display for ThingDescContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "object_type_code: [Thing], mac_address: [{}], owner-depend-id: [{}]", hex::encode_upper(self.mac_address()), self.owner_depend_id())
    }
}

impl ObjectDescTrait for ThingDescContent {
    fn object_type_code(&self) -> ObjectTypeCode {
        ObjectTypeCode::Thing
    }

    type OwnerObj = ObjectId;
    type AreaObj = Area;
    type AuthorObj = ObjectId;
    type PublicKeyObj = PublicKey;

}

impl Serialize for ThingDescContent {
    fn raw_capacity(&self) -> usize {
        self.mac_address.raw_capacity() + 
        self.owner_depend_id.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.mac_address.serialize(buf)?;
        let buf = self.owner_depend_id.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for ThingDescContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (mac_address, buf) = Vec::<u8>::deserialize(buf)?;
        let (owner_depend_id, buf) = String::deserialize(buf)?;

        Ok((Self{
            mac_address: {
                let mut v = [0u8; 6];
                v.clone_from_slice(&mac_address.as_slice()[0..6]);
                v
            },
            owner_depend_id,
        }, buf))
    }

}

#[derive(Clone, Default)]
pub struct ThingBodyContent {
    name: String,
    user_data: HashMap<String, String>,
}

impl ThingBodyContent {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn user_data(&self) -> &HashMap<String, String> {
        &self.user_data
    }

    pub fn set_userdata(&mut self, user_data: HashMap<String, String>) {
        self.user_data = user_data;
    }

    pub fn take_userdata(&mut self) -> HashMap<String, String> {
        std::mem::replace(&mut self.user_data, Default::default())
    }

    pub fn mut_user_data(&mut self) -> &mut HashMap<String, String> {
        &mut self.user_data
    }
}

impl std::fmt::Display for ThingBodyContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: [{:?}], user_data count: [{}]", 
            self.name,
            self.user_data.len())
    }
}

impl ObjectBodyTrait for ThingBodyContent { 
}

impl Serialize for ThingBodyContent {
    fn raw_capacity(&self) -> usize {
        self.name.raw_capacity() + 
        self.user_data.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.name.serialize(buf)?;
        let buf = self.user_data.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for ThingBodyContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (name, buf) = String::deserialize(buf)?;
        let (user_data, buf) = HashMap::<String, String>::deserialize(buf)?;

        Ok((Self{
            name, user_data,
        }, buf))
    }

}
