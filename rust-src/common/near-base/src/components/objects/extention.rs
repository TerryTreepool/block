
use crate::{components::Area, 
            {Serialize, Deserialize},
            errors::*, public_key::PublicKey, };

use super::{object_type::{ObjectId, ObjectTypeCode},
            object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody},
            object_builder::{ObjectDescTrait, ObjectBodyTrait}};

pub type ExtentionDesc = NamedObjectDesc<ExtentionDescContent>;
pub type ExtentionBody = NamedObjectBody<ExtentionBodyContent>;
pub type ExtentionObject = NamedObject<ExtentionDescContent, ExtentionBodyContent>;

#[derive(Clone)]
pub struct ExtentionDescContent {
    object_type_code: ObjectTypeCode,
    extention_name: String,
}

impl ExtentionDescContent {
    pub fn set_extention_name(&mut self, name: impl std::string::ToString) -> &mut Self {
        self.extention_name = name.to_string();
        self
    }

    pub fn get_extention_name(&self) -> &str {
        self.extention_name.as_str()
    }
}

impl std::fmt::Display for ExtentionDescContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "object_type_code: [{}], extention_name: [{}]",
               self.object_type_code,
               self.extention_name,
        )
    }
}

impl ObjectDescTrait for ExtentionDescContent {
    fn object_type_code(&self) -> ObjectTypeCode {
        self.object_type_code
    }

    type OwnerObj = ObjectId;
    type AreaObj = Area;
    type AuthorObj = ObjectId;
    type PublicKeyObj = PublicKey;

}

impl std::default::Default for ExtentionDescContent {
    fn default() -> Self {
        Self {
            object_type_code: ObjectTypeCode::Extention,
            extention_name: String::default(),
        }        
    }
}

impl Serialize for ExtentionDescContent {
    fn raw_capacity(&self) -> usize {
        self.object_type_code.raw_capacity() +
        self.extention_name.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.object_type_code.serialize(buf)?;
        let buf = self.extention_name.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for ExtentionDescContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (object_type_code, buf) = ObjectTypeCode::deserialize(buf)?;
        let (extention_name, buf) = String::deserialize(buf)?;

        Ok((Self{
            object_type_code, extention_name
        }, buf))
    }
}

#[derive(Clone)]
pub struct ExtentionBodyContent {
    subscribe_messages: Vec<String>,
}

impl ExtentionBodyContent {
    pub fn set_subscribe_message(&mut self, topic: impl std::string::ToString) -> &mut Self {
        self.subscribe_messages.push(topic.to_string());
        self
    }

    pub fn set_subscribe_message_group(&mut self, command: &[impl std::string::ToString]) -> &mut Self {
        self.subscribe_messages.clear();
        command.iter().for_each(| c | self.subscribe_messages.push(c.to_string().to_ascii_lowercase()) );
        self
    }

    pub fn subscribe_messages(&self) -> &Vec<String> {
        &self.subscribe_messages
    }

}

impl std::fmt::Display for ExtentionBodyContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "subscribe_messages: [{:?}]", self.subscribe_messages)
    }
}

impl std::default::Default for ExtentionBodyContent {
    fn default() -> Self {
        Self {
            subscribe_messages: vec![],
        }
    }
}

impl ObjectBodyTrait for ExtentionBodyContent {}

impl Serialize for ExtentionBodyContent {
    fn raw_capacity(&self) -> usize {
        self.subscribe_messages.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.subscribe_messages.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for ExtentionBodyContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (subscribe_messages, buf) = Vec::<String>::deserialize(buf)?;

        Ok((Self{
            subscribe_messages,
        }, buf))
    }

}

// #[cfg(test)]
// mod test {

//     #[test]
//     fn test_file_manager_object() {
//         use std::path::PathBuf;
//         use crate::{ObjectBuilder, extention::{ExtentionDescContent, ExtentionBodyContent}, Serialize, ExtentionObject, Deserialize, FileEncoder};

//         let service = 
//             ObjectBuilder::new(ExtentionDescContent::default()
//                                         .set_extention_name("file_manager")
//                                         .subscribe_message("/test1/test1"),
//                                ExtentionBodyContent::default()
//                                         .set_name(Some("file manager")))
//                 .update_desc(|_desc| {
//                 })
//                 .update_body(|_body| {
//                 })
//                 .build().unwrap();

//         println!("service-id={}", service.object_id().to_string());
//         let mut b = [0u8;1024];

//         if let Err(err) = service.encode_to_file(PathBuf::new().join("file-manager.desc").as_path(), false) {
//             println!("failed encode-to-file with {}", err);
//         }

//         {
//             let b_end = service.serialize(&mut b).unwrap();
//             let len = 1024 - b_end.len();

//             println!("{}", len);
//             println!("{:?}", b);
//         }

//         {
//             let (r, _) = ExtentionObject::deserialize(&b).unwrap();

//             println!("service-id={}", r.object_id().to_string());

//         }
//     }

//     #[test]
//     fn test_gateway_object() {
//         use std::path::PathBuf;
//         use crate::{ObjectBuilder, extention::{ExtentionDescContent, ExtentionBodyContent}, Serialize, ExtentionObject, Deserialize, FileEncoder};

//         let service = 
//             ObjectBuilder::new(ExtentionDescContent::default()
//                                         .set_extention_name("gateway-n"),
//                                ExtentionBodyContent::default()
//                                         .set_name(Some("gateway runtime")))
//                 .update_desc(|_desc| {
//                 })
//                 .update_body(|_body| {
//                 })
//                 .build().unwrap();

//         println!("service-id={}", service.object_id().to_string());
//         let mut b = [0u8;1024];

//         if let Err(err) = service.encode_to_file(PathBuf::new().join("gateway-n.desc").as_path(), false) {
//             println!("failed encode-to-file with {}", err);
//         }

//         {
//             let b_end = service.serialize(&mut b).unwrap();
//             let len = 1024 - b_end.len();

//             println!("{}", len);
//             println!("{:?}", b);
//         }

//         {
//             let (r, _) = ExtentionObject::deserialize(&b).unwrap();

//             println!("service-id={}", r.object_id().to_string());

//         }
//     }

//     #[test]
//     fn test_hci_object() {
//         use std::path::PathBuf;
//         use crate::{ObjectBuilder, extention::{ExtentionDescContent, ExtentionBodyContent}, Serialize, ExtentionObject, Deserialize, FileEncoder};

//         let service = 
//             ObjectBuilder::new(ExtentionDescContent::default()
//                                         .set_extention_name("hci-runtime"),
//                                ExtentionBodyContent::default()
//                                         .set_name(Some("hci runtime")))
//                 .update_desc(|_desc| {
//                 })
//                 .update_body(|_body| {
//                 })
//                 .build().unwrap();

//         println!("service-id={}", service.object_id().to_string());
//         let mut b = [0u8;1024];

//         if let Err(err) = service.encode_to_file(PathBuf::new().join("hci-service.desc").as_path(), false) {
//             println!("failed encode-to-file with {}", err);
//         }

//         {
//             let b_end = service.serialize(&mut b).unwrap();
//             let len = 1024 - b_end.len();

//             println!("{}", len);
//             println!("{:?}", b);
//         }

//         {
//             let (r, _) = ExtentionObject::deserialize(&b).unwrap();

//             println!("service-id={}", r.object_id().to_string());

//         }
//     }

//     #[test]
//     fn test_hci_manager() {
//         use std::path::PathBuf;
//         use crate::{ObjectBuilder, extention::{ExtentionDescContent, ExtentionBodyContent}, Serialize, ExtentionObject, Deserialize, FileEncoder};

//         let service = 
//             ObjectBuilder::new(ExtentionDescContent::default()
//                                         .set_extention_name("hci-manager"),
//                                ExtentionBodyContent::default()
//                                         .set_name(Some("hci manager")))
//                 .update_desc(|_desc| {
//                 })
//                 .update_body(|_body| {
//                 })
//                 .build().unwrap();

//         println!("service-id={}", service.object_id().to_string());
//         let mut b = [0u8;1024];

//         if let Err(err) = service.encode_to_file(PathBuf::new().join("hci-manager.desc").as_path(), false) {
//             println!("failed encode-to-file with {}", err);
//         }

//         {
//             let b_end = service.serialize(&mut b).unwrap();
//             let len = 1024 - b_end.len();

//             println!("{}", len);
//             println!("{:?}", b);
//         }

//         {
//             let (r, _) = ExtentionObject::deserialize(&b).unwrap();

//             println!("service-id={}", r.object_id().to_string());

//         }
//     }

// }


