

use crate::{components::Area, 
            {Serialize, Deserialize},
            errors::*, public_key::PublicKey, };

use super::{object_type::{ObjectId, ObjectTypeCode},
            object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody},
            object_builder::{ObjectDescTrait, ObjectBodyTrait}};

pub type PeopleDesc = NamedObjectDesc<PeopleDescContent>;
pub type PeopleBody = NamedObjectBody<PeopleBodyContent>;
pub type PeopleObject = NamedObject<PeopleDescContent, PeopleBodyContent>;

#[derive(Clone, Default)]
pub struct PeopleDescContent {
}

impl PeopleDescContent {
    pub fn new() -> Self {
        Self{ }
    }
}

impl std::fmt::Display for PeopleDescContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "desc null", )
    }
}

impl ObjectDescTrait for PeopleDescContent {
    fn object_type_code(&self) -> ObjectTypeCode {
        ObjectTypeCode::People
    }

    type OwnerObj = ObjectId;
    type AreaObj = Area;
    type AuthorObj = ObjectId;
    type PublicKeyObj = PublicKey;

}

impl Serialize for PeopleDescContent {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        Ok(buf)
    }

}

impl Deserialize for PeopleDescContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        Ok((Self{}, buf))
    }
}

#[derive(Clone, Default)]
pub struct PeopleBodyContent {
    core_service_list: Vec<ObjectId>,
    name: Option<String>,
    userdata: Option<Vec<u8>>,
}

impl PeopleBodyContent {
    pub fn set_core_service_list(&mut self, list: Vec<ObjectId>) -> &mut Self {
        self.core_service_list = list;
        self
    }

    pub fn set_name(&mut self, name: Option<impl std::string::ToString>) -> &mut Self {
        self.name = name.map(| data | data.to_string());
        self
    }

    pub fn set_userdata(&mut self, userdata: Option<Vec<u8>>) -> &mut Self {
        self.userdata = userdata;
        self
    }

    pub fn core_service_list(&self) -> &[ObjectId] {
        &self.core_service_list
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(| v | v.as_str())
    }

    pub fn userdata(&self) -> Option<&[u8]> {
        self.userdata.as_ref().map(| data | data.as_slice())
    }

}

impl std::fmt::Display for PeopleBodyContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: [{:?}]", self.name)
    }
}

impl ObjectBodyTrait for PeopleBodyContent {}

impl Serialize for PeopleBodyContent {
    fn raw_capacity(&self) -> usize {
        self.core_service_list.raw_capacity() +
        self.name.raw_capacity() +
        self.userdata.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.core_service_list.serialize(buf)?;
        let buf = self.name.serialize(buf)?;
        let buf = self.userdata.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for PeopleBodyContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (core_service_list, buf) = Vec::<ObjectId>::deserialize(buf)?;
        let (name, buf) = Option::<String>::deserialize(buf)?;
        let (userdata, buf) = Option::<Vec<u8>>::deserialize(buf)?;

        Ok((Self{
            core_service_list,
            name,
            userdata,
        }, buf))
    }

}


