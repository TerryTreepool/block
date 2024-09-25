

use crate::{Area, Serialize, Deserialize, PublicKey, hash_data,};
use crate::errors::*;
use crate::time::*;

use super::object_impl::NamedObject;
use super::{object_impl::{NamedObjectDesc, NamedObjectBody}, 
            object_type::{ObjectTypeCode, ObjectId}};

/// object desc struct & builder
pub trait ObjectDescTrait: std::fmt::Display + Clone {

    fn object_type_code(&self) -> ObjectTypeCode;
    fn version(&self) -> u8 {
        0
    }

    type OwnerObj: Into<ObjectId> + Clone + Serialize + Deserialize + std::fmt::Debug;
    type AreaObj: Into<Area> + Clone + Serialize + Deserialize + std::fmt::Debug;
    type AuthorObj: Into<ObjectId> + Clone + Serialize + Deserialize + std::fmt::Debug;
    type PublicKeyObj: Into<PublicKey> + Clone + Serialize + Deserialize + std::fmt::Debug;
}

pub trait ObjectBodyTrait: std::fmt::Display + Clone {
    fn version(&self) -> u8 {
        0
    }

}

pub struct ObjectDescBuilder<T: ObjectDescTrait> {
    create_timestamp: u64,      // unix timestamp
    expired_time: Option<u64>,  // unix timestamp

    owner: Option<T::OwnerObj>,
    area: Option<T::AreaObj>,
    author: Option<T::AuthorObj>,
    public_key: Option<T::PublicKeyObj>,

    desc: T
}

impl<T: ObjectDescTrait + Serialize + Deserialize + std::default::Default> ObjectDescBuilder<T> {
    pub fn new(desc: T) -> Self {
        Self {
            create_timestamp: now(),
            expired_time: None,
            owner: None,
            area: None,
            author: None,
            public_key: None,
            desc
        }
    }

    pub fn no_create_time(&mut self) {
        self.create_timestamp = 0;
        self.expired_time = None;
    }

    pub fn set_create_timestamp(&mut self, create_timestamp: u64) {
        self.create_timestamp = create_timestamp;
    }

    pub fn set_expired_time(&mut self, expired_time: Option<u64>) {
        self.expired_time = expired_time;
    }

    pub fn set_owner(&mut self, owner: Option<T::OwnerObj>) {
        self.owner = owner;
    }

    pub fn set_area(&mut self, area: Option<T::AreaObj>) {
        self.area = area;
    }

    pub fn set_author(&mut self, author: Option<T::AuthorObj>) {
        self.author = author;
    }

    pub fn set_public_key(&mut self, key: T::PublicKeyObj) {
        self.public_key = Some(key);
    }

    pub fn set_desc(&mut self, desc: T) {
        self.desc = desc;
    }

    pub fn create_timestamp(&self) -> u64 {
        self.create_timestamp
    }

    pub fn expired_time(&self) -> Option<u64> {
        self.expired_time
    }

    pub fn owner(&self) -> &Option<T::OwnerObj> {
        &self.owner
    }

    pub fn area(&self) -> &Option<T::AreaObj> {
        &self.area
    }

    pub fn author(&self) -> &Option<T::AuthorObj> {
        &self.author
    }

    pub fn public_key(&self) -> &Option<T::PublicKeyObj> {
        &self.public_key
    }

    pub fn desc(&self) -> &T {
        &self.desc
    }

    pub fn mut_desc(&mut self) -> &mut T {
        &mut self.desc
    }

    pub fn build(&self) -> NamedObjectDesc<T> {
        NamedObjectDesc::build(self)
    }
}

pub struct ObjectBodyBuilder<T: ObjectBodyTrait + Serialize + Deserialize> {
    body: T,
}

impl<T: ObjectBodyTrait + Serialize + Deserialize + std::default::Default> ObjectBodyBuilder<T> {
    pub fn new(body: T) -> Self {
        Self {
            body
        }
    }

    pub fn set_body(&mut self, body: T) {
        self.body = body;
    }

    pub fn body(&self) -> &T {
        &self.body
    }

    pub fn mut_body(&mut self) -> &mut T {
        &mut self.body
    }

    pub fn build(&self) -> NamedObjectBody<T> {
        NamedObjectBody::build(&self)
    }
}

struct ObjectIdBuilder<'a, T: ObjectDescTrait + Serialize + Deserialize + std::default::Default> {
    ptr: &'a NamedObjectDesc<T>,
    area: Option<Area>,
    is_owner: bool,
    is_author: bool,
    is_publickey: bool,
}

impl<'a, T> ObjectIdBuilder<'a, T>
where T: ObjectDescTrait + Serialize + Deserialize + std::default::Default {
    fn new(ptr: &'a NamedObjectDesc<T>) -> Self {
        Self {
            ptr: ptr,
            area: {
                if let Some(area) = ptr.area() {
                    let area = area.clone().into();
                    Some(area)
                } else {
                    None
                }
            },
            is_owner: ptr.owner().is_some(),
            is_author: ptr.author().is_some(),
            is_publickey: ptr.public_key().is_some(),
        }
    }

    fn build(self) -> NearResult<ObjectId> {
        let mut h = {
            let size = self.ptr.raw_capacity();
            let mut desc_buf = vec![0u8; size];
            let remain_buf = self.ptr.serialize(&mut desc_buf)?;
            let len = size - remain_buf.len();
            hash_data(&desc_buf[..len])
        };

        let features = h.as_mut_slice();
        features[0] = 0;
        if self.area.is_some() {
            features[0] |= 0b_0000_0001;
        }
        if self.is_owner {
            features[0] |= 0b_0000_0010;
        }
        if self.is_author {
            features[0] |= 0b_0000_0100;
        }
        if self.is_publickey {
            features[0] |= 0b_0000_1000;
        }

        let (type_master, type_property) = self.ptr.object_type_code().split();
        features[0] = type_master << 4 | features[0];
        features[1] = type_property;

        if let Some(area) = self.area {
            let val: u64 = area.into();
            let val_len = std::mem::size_of_val(&val);
            unsafe {
                std::ptr::copy(val.to_be_bytes().as_ptr(), features[2..].as_mut_ptr(), val_len);
            }
        }

        Ok(ObjectId::from(h.as_ref()))
    }
}

pub struct ObjectBuilder<DESC, BODY>
where DESC: ObjectDescTrait + Serialize + Deserialize,
      BODY: ObjectBodyTrait + Serialize + Deserialize {
    desc_builder: ObjectDescBuilder<DESC>,
    body_builder: ObjectBodyBuilder<BODY>,
}

impl<DESC, BODY> ObjectBuilder<DESC, BODY> 
where DESC: ObjectDescTrait + Serialize + Deserialize + std::default::Default,
      BODY: ObjectBodyTrait + Serialize + Deserialize + std::default::Default {
    pub fn new(desc: DESC, body: BODY) -> Self {
        Self {
            desc_builder: ObjectDescBuilder::new(desc),
            body_builder: ObjectBodyBuilder::new(body),
        }
    }

    pub fn update_desc<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut ObjectDescBuilder<DESC>) {
        f(&mut self.desc_builder);
        self
    }

    pub fn update_body<F>(mut self, f: F) -> Self
    where 
        F: FnOnce(&mut ObjectBodyBuilder<BODY>) {
        f(&mut self.body_builder);
        self
    }

    pub fn mut_desc_builder(&mut self) -> &mut ObjectDescBuilder<DESC> {
        &mut self.desc_builder
    }

    pub fn body_builder(&self) -> &ObjectBodyBuilder<BODY> {
        &self.body_builder
    }

    pub fn mut_body_builder(&mut self) -> &mut ObjectBodyBuilder<BODY> {
        &mut self.body_builder
    }

    pub fn build(self) -> NearResult<NamedObject<DESC, BODY>> {
        let desc = self.desc_builder.build();
        let body = self.body_builder.build();
        let id = ObjectIdBuilder::new(&desc)
                                            .build()?;

        Ok(NamedObject::<DESC, BODY>::build(id, desc, body))
    }
}
