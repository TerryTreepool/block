

use crate::components::Hash256;
use crate::{Area, Serialize, Deserialize, PublicKey,};
use crate::errors::*;
use crate::time::*;

// use super::object_impl::NamedObject;
use super::{/* object_impl::{NamedObjectDesc, NamedObjectBody}, */
            object_type::{ObjectTypeCode, ObjectId}};
// use super::{ServiceObject, DeviceObject, ThingObject};

/// object desc struct & builder
pub trait ObjectDescTrait {

    fn object_type_code(&self) -> ObjectTypeCode;
    fn version(&self) -> u8 {
        0
    }

    type OwnerObj: Into<ObjectId> + Clone + Serialize + Deserialize;
    type AreaObj: Into<Area> + Clone + Serialize + Deserialize;
    type AuthorObj: Into<ObjectId> + Clone + Serialize + Deserialize;
    type PublicKeyObj: Into<PublicKey> + Clone + Serialize + Deserialize;
}

pub trait ObjectBodyTrait {
    fn version(&self) -> u8 {
        0
    }

}

// pub trait ObjectTrait {
//     fn id(&self) -> &ObjectId;
//     fn as_service_object(&self) -> Option<ServiceObject> { None }
//     fn as_device_object(&self) -> Option<DeviceObject> { None }
//     fn as_thing_object(&self) -> Option<ThingObject> { None }
// }

// pub struct ObjectDescBuilder<T: ObjectDescTrait> {
//     create_timestamp: u64,      // unix timestamp
//     expired_time: Option<u64>,  // unix timestamp

//     owner: Option<T::OwnerObj>,
//     area: Option<T::AreaObj>,
//     author: Option<T::AuthorObj>,
//     public_key: Option<T::PublicKeyObj>,

//     desc: T
// }

// impl<T: ObjectDescTrait + Serialize + Deserialize> ObjectDescBuilder<T> {
//     pub fn new(desc: T) -> Self {
//         Self {
//             create_timestamp: now(),
//             expired_time: None,
//             owner: None,
//             area: None,
//             author: None,
//             public_key: None,
//             desc
//         }
//     }

//     pub fn set_expired_time(mut self, expired_time: Option<u64>) -> Self {
//         self.expired_time = expired_time;
//         self
//     }

//     pub fn set_owner(&mut self, owner: T::OwnerObj) {
//         self.owner = Some(owner);
//     }

//     pub fn set_area(&mut self, area: T::AreaObj) {
//         self.area = Some(area);
//     }

//     pub fn set_author(&mut self, author: T::AuthorObj) {
//         self.author = Some(author);
//     }

//     pub fn set_public_key(&mut self, key: T::PublicKeyObj) {
//         self.public_key = Some(key);
//     }

//     pub fn set_desc(&mut self, desc: T) {
//         self.desc = desc;
//     }

//     pub fn create_timestamp(&self) -> u64 {
//         self.create_timestamp
//     }

//     pub fn expired_time(&self) -> Option<u64> {
//         self.expired_time
//     }

//     pub fn owner(&self) -> &Option<T::OwnerObj> {
//         &self.owner
//     }

//     pub fn area(&self) -> &Option<T::AreaObj> {
//         &self.area
//     }

//     pub fn author(&self) -> &Option<T::AuthorObj> {
//         &self.author
//     }

//     pub fn public_key(&self) -> &Option<T::PublicKeyObj> {
//         &self.public_key
//     }

//     pub fn desc(&self) -> &T {
//         &self.desc
//     }

//     pub fn build(&self) -> NamedObjectDesc<T> {
//         NamedObjectDesc::build(self)
//     }
// }

// pub struct ObjectBodyBuilder<T: ObjectBodyTrait + Serialize + Deserialize> {
//     body: T,
// }

// impl<T: ObjectBodyTrait + Serialize + Deserialize> ObjectBodyBuilder<T> {
//     pub fn new(body: T) -> Self {
//         Self {
//             body
//         }
//     }

//     pub fn set_body(&mut self, body: T) {
//         self.body = body;
//     }

//     pub fn body(&self) -> &T {
//         &self.body
//     }

//     pub fn build(&self) -> NamedObjectBody<T> {
//         NamedObjectBody::build(&self)
//     }
// }

// struct ObjectIdBuilder<'a, T: ObjectDescTrait + Serialize + Deserialize> {
//     ptr: &'a NamedObjectDesc<T>,
//     area: Option<Area>,
//     is_owner: bool,
//     is_author: bool,
//     is_publickey: bool,
// }

// impl<'a, T> ObjectIdBuilder<'a, T>
// where T: ObjectDescTrait + Serialize + Deserialize {
//     fn new(ptr: &'a NamedObjectDesc<T>) -> Self {
//         Self {
//             ptr: ptr,
//             area: {
//                 if let Some(area) = ptr.area() {
//                     let area = area.clone().into();
//                     Some(area)
//                 } else {
//                     None
//                 }
//             },
//             is_owner: ptr.owner().is_some(),
//             is_author: ptr.author().is_some(),
//             is_publickey: ptr.public_key().is_some(),
//         }
//     }

//     fn build(self) -> NearResult<ObjectId> {
//         let mut h = {
//             let mut desc_buf = [0u8; 1024];
//             let desc_buf_len = 1024usize;
//             let remain_buf = self.ptr.serialize(&mut desc_buf)?;
//             let len = desc_buf_len - remain_buf.len();
//             Hash256::new(&desc_buf[..len])
//         };

//         let mut features = [0u8; 10];
//         let freaures_size = features.len();

//         if self.area.is_some() {
//             features[0] |= 0b_0000_0001;
//         }
//         if self.is_owner {
//             features[0] |= 0b_0000_0010;
//         }
//         if self.is_author {
//             features[0] |= 0b_0000_0100;
//         }
//         if self.is_publickey {
//             features[0] |= 0b_0000_1000;
//         }

//         if let Some(area) = self.area {
//             features[2..freaures_size].copy_from_slice(&{
//                 let val: u64 = area.into();
//                 val
//             }.to_be_bytes());
//         }

//         let h_buf = h.as_mut_slice();
//         h_buf[..freaures_size].copy_from_slice(&features);

//         Ok(ObjectId::from(h.as_ref()))
//     }
// }

// pub struct ObjectBuilder<DESC, BODY>
// where DESC: ObjectDescTrait + Serialize + Deserialize,
//       BODY: ObjectBodyTrait + Serialize + Deserialize {
//     desc_builder: ObjectDescBuilder<DESC>,
//     body_builder: ObjectBodyBuilder<BODY>,
// }

// impl<DESC, BODY> ObjectBuilder<DESC, BODY> 
// where DESC: ObjectDescTrait + Serialize + Deserialize,
//       BODY: ObjectBodyTrait + Serialize + Deserialize {
//     pub fn new(desc: DESC, body: BODY) -> Self {
//         Self {
//             desc_builder: ObjectDescBuilder::new(desc),
//             body_builder: ObjectBodyBuilder::new(body),
//         }
//     }

//     pub fn update_desc<F>(mut self, f: F) -> Self
//     where
//         F: FnOnce(&mut ObjectDescBuilder<DESC>) {
//         f(&mut self.desc_builder);
//         self
//     }

//     pub fn update_body<F>(mut self, f: F) -> Self
//     where 
//         F: FnOnce(&mut ObjectBodyBuilder<BODY>) {
//         f(&mut self.body_builder);
//         self
//     }

//     pub fn mut_desc_builder(&mut self) -> &mut ObjectDescBuilder<DESC> {
//         &mut self.desc_builder
//     }

//     pub fn body_builder(&self) -> &ObjectBodyBuilder<BODY> {
//         &self.body_builder
//     }

//     pub fn mut_body_builder(&mut self) -> &mut ObjectBodyBuilder<BODY> {
//         &mut self.body_builder
//     }

//     pub fn build(&self) -> NearResult<NamedObject<DESC, BODY>> {
//         let desc = self.desc_builder.build();
//         let body = self.body_builder.build();
//         let id = ObjectIdBuilder::new(&desc);

//         Ok(NamedObject::<DESC, BODY>::build(id.build()?, desc, body))
//     }
// }
