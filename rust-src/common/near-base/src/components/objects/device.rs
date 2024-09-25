
use crate::{components::ObjectTypeCode, errors::*, public_key::PublicKey, Area, Deserialize, Endpoint, EndpointPair, ObjectBodyTrait, ObjectDescTrait, Serialize };

use super::{object_type::ObjectId,
            object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody},
    };

pub type DeviceDesc = NamedObjectDesc<DeviceDescContent>;
pub type DeviceBody = NamedObjectBody<DeviceBodyContent>;
pub type DeviceObject = NamedObject<DeviceDescContent, DeviceBodyContent>;
pub type DeviceId = ObjectId;

#[derive(Clone, Default)]
pub struct DeviceDescContent {
    object_type_code: ObjectTypeCode,
    id: ObjectId,
}

impl DeviceDescContent {
    pub fn with_device(code: u8) -> Self {
        Self {
            object_type_code: ObjectTypeCode::Device(code), 
            id: ObjectId::default(),
        }
    }

    pub fn with_service(code: u8) -> Self {
        Self {
            object_type_code: ObjectTypeCode::Service(code), 
            id: ObjectId::default(),
        }
    }

    pub fn with_thing() -> Self {
        Self {
            object_type_code: ObjectTypeCode::Thing, 
            id: ObjectId::default(),
        }
    }

    pub fn id(&self) -> &ObjectId {
        &self.id
    }
}

impl std::fmt::Display for DeviceDescContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "object_type_code: [{}], id: [{}]", self.object_type_code, self.id)
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
        self.object_type_code.raw_capacity() + 
        self.id.raw_capacity()
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
            object_type_code, id, 
        }, buf))
    }

}

#[derive(Clone)]
pub struct DeviceBodyContent {
    endpoints: Vec<Endpoint>,
    reverse_endpoint_array: Vec<EndpointPair>,
    stun_node_list: Vec<ObjectId>,
    turn_node_list: Vec<ObjectId>,
    user_data: Vec<u8>,
    name: Option<String>,
}

impl std::default::Default for DeviceBodyContent {
    fn default() -> Self {
        Self {
            endpoints: vec![], 
            reverse_endpoint_array: vec![],
            stun_node_list: vec![], 
            turn_node_list: vec![], 
            user_data: vec![],
            name: None
        }
    }
}

impl DeviceBodyContent {
    pub fn set_endpoints(&mut self, endpoints: Vec<Endpoint>) {
        self.endpoints = endpoints;
    }

    pub fn set_reverse_endpoint_array(&mut self, endpoints_array: Vec<EndpointPair>) {
        self.reverse_endpoint_array = endpoints_array;
    }

    pub fn set_stun_node_list(&mut self, stun_node_list: Vec<ObjectId>) {
        self.stun_node_list = stun_node_list;
    }

    pub fn set_turn_node_list(&mut self, turn_node_list: Vec<ObjectId>) {
        self.turn_node_list = turn_node_list;
    }

    pub fn set_userdata(&mut self, user_data: Vec<u8>) {
        self.user_data = user_data;
    }

    pub fn set_name(&mut self, name: Option<impl std::string::ToString>) {
        self.name = name.map(| text | text.to_string() );
    }

    pub fn endpoints(&self) -> &Vec<Endpoint> {
        &self.endpoints
    }

    pub fn reverse_endpoint_array(&self) -> &Vec<EndpointPair> {
        &self.reverse_endpoint_array
    }

    pub fn stun_node_list(&self) -> &Vec<ObjectId> {
        &self.stun_node_list
    }

    pub fn turn_node_list(&self) -> &Vec<ObjectId> {
        &self.turn_node_list
    }

    pub fn userdata(&self) -> &Vec<u8> {
        &self.user_data
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref()
            .map(| name | name.as_str() )
    }

    pub fn mut_endpoints(&mut self) -> &mut Vec<Endpoint> {
        &mut self.endpoints
    }

    pub fn mut_reverse_endpoint_array(&mut self) -> &mut Vec<EndpointPair> {
        &mut self.reverse_endpoint_array
    }

    pub fn mut_stun_node_list(&mut self) -> &mut Vec<ObjectId> {
        &mut self.stun_node_list
    }

    pub fn mut_turn_node_list(&mut self) -> &mut Vec<ObjectId> {
        &mut self.turn_node_list
    }

    pub fn update_name(&mut self, name: Option<String>) {
        self.name = name;
    }

}

impl std::fmt::Display for DeviceBodyContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "endpoints: [{:?}], reverse_endpoint_array:[{:?}], stun_node_list: [{:?}], turn_node_list: [{:?}], userdata: [{}], name: [{:?}]", 
               self.endpoints,
               self.reverse_endpoint_array,
               self.stun_node_list,
               self.turn_node_list,
               self.user_data.len(),
               self.name)
    }
}

impl ObjectBodyTrait for DeviceBodyContent { 
}

impl Serialize for DeviceBodyContent {
    fn raw_capacity(&self) -> usize {
        self.endpoints.raw_capacity() + 
        self.reverse_endpoint_array.raw_capacity() +
        self.stun_node_list.raw_capacity() +
        self.turn_node_list.raw_capacity() +
        self.user_data.raw_capacity() +
        self.name.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.endpoints.serialize(buf)?;
        let buf = self.reverse_endpoint_array.serialize(buf)?;
        let buf = self.stun_node_list.serialize(buf)?;
        let buf = self.turn_node_list.serialize(buf)?;
        let buf = self.user_data.serialize(buf)?;

        let buf = self.name.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for DeviceBodyContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (endpoints, buf) = Vec::<Endpoint>::deserialize(buf)?;
        let (reverse_endpoint_array, buf) = Vec::<EndpointPair>::deserialize(buf)?;
        let (stun_node_list, buf) = Vec::<ObjectId>::deserialize(buf)?;
        let (turn_node_list, buf) = Vec::<ObjectId>::deserialize(buf)?;
        let (user_data, buf) = Vec::<u8>::deserialize(buf)?;

        let (name, buf) = Option::<String>::deserialize(buf)?;

        Ok((Self{
            endpoints, reverse_endpoint_array,
            stun_node_list, turn_node_list,
            user_data,
            name
        }, buf))
    }

}

// #[cfg(test)]
// mod test {

//     use std::path::PathBuf;

//     use crate::{Area, builder_codec::FileEncoder, extention::{ExtentionDescContent, ExtentionBodyContent},};

//     use super::*;
//     use crate::ObjectBuilder;

//     #[test]
//     fn test_service() {
//         let service = 
//             ObjectBuilder::new(DeviceDescContent::with_device(1), 
//                                DeviceBodyContent::default())
//                 .update_desc(|desc| {
//                     desc.set_area(Area::new(123, 10, 2321, 10));
//                 })
//                 .update_body(|_body| {
//                     _body.mut_body().set_name(Some("core-service"));
//                 })
//                 .build().unwrap();

//         println!("{}", service.object_id().object_type_code().unwrap());

//         println!("core-servic: {}", service);

//         println!("{:?}", service.encode_to_file(PathBuf::new().join("core-service.desc").as_path(), false));
//     }

//     #[test]
//     fn test_runtime() {
//         let runtime = 
//             ObjectBuilder::new(ExtentionDescContent::default().set_extention_name("test").subscribe_message("/data/1000").subscribe_message("/data/2000"),
//                                ExtentionBodyContent::default().set_name(Some("runtime by test")))
//                 .update_desc(| desc | {
//                     desc.set_area(Area::new(123, 10, 2321, 10));
//                 })
//                 .update_body(| _body | {
//                 })
//                 .build().unwrap();

//         println!("runtime: {}", runtime);

//         if let Err(err) = runtime.encode_to_file(PathBuf::new().join("runtime.desc").as_path(), false) {
//             println!("failed encode-to-file with {}", err);
//         }
//     }

//     #[test]
//     fn file_manager_runtime() {
//         let runtime = 
//             ObjectBuilder::new(ExtentionDescContent::default().set_extention_name("file_manager_runtime"),
//                                ExtentionBodyContent::default().set_name(Some("runtime by file's manager")))
//                 .update_desc(| desc | {
//                     desc.set_area(Area::new(123, 10, 2321, 10));
//                 })
//                 .update_body(| _body | {
//                 })
//                 .build().unwrap();

//         println!("runtime: {}", runtime);

//         if let Err(err) = runtime.encode_to_file(PathBuf::new().join("file_manager_runtime.desc").as_path(), false) {
//             println!("failed encode-to-file with {}", err);
//         }
//     }

// }


