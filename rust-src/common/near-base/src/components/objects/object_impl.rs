
use crate::{Serialize, Deserialize, now, };
use crate::errors::*;

use super::{object_type::{ObjectId, ObjectTypeCode},
            object_builder::{ObjectDescBuilder, ObjectBodyBuilder, 
                             ObjectDescTrait, ObjectBodyTrait}};

/// named
/// object desc content
#[derive(Clone)]
pub struct NamedObjectDesc<T: ObjectDescTrait + Serialize + Deserialize + std::default::Default> {
    create_timestamp: u64,
    expired_time: Option<u64>,

    owner: Option<T::OwnerObj>,
    area: Option<T::AreaObj>,
    author: Option<T::AuthorObj>,
    public_key: Option<T::PublicKeyObj>,

    content: T,
}

impl<T: ObjectDescTrait + Serialize + Deserialize + std::default::Default> std::default::Default for NamedObjectDesc<T> {
    fn default() -> Self {
        Self {
            create_timestamp: 0u64,
            expired_time: None,
            owner: None,
            area: None,
            author: None,
            public_key: None,
            content: T::default(),
        }
    }
}

impl<T: ObjectDescTrait + Serialize + Deserialize + std::default::Default> std::fmt::Display for NamedObjectDesc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "create_timestamp: [{}], expired_time: [{:?}], owner: [{:?}], area: [{:?}], author: [{:?}], public_key: [{:?}], content: [{}]",
               self.create_timestamp,
               self.expired_time,
               self.owner,
               self.area,
               self.author,
               self.public_key,
               self.content
            )
    }
}

impl<T: ObjectDescTrait + Serialize + Deserialize + std::default::Default> NamedObjectDesc<T> {
    pub fn build(builder: &ObjectDescBuilder<T>) -> Self {
        Self {
            create_timestamp: builder.create_timestamp(),
            expired_time: builder.expired_time(),
            owner: {
                if let Some(o) = builder.owner() {
                    Some(o.clone())
                } else {
                    None
                }
            },
            area: {
                if let Some(o) = builder.area() {
                    Some(o.clone())
                } else {
                    None
                }
            },
            author: {
                if let Some(o) = builder.author() {
                    Some(o.clone())
                } else {
                    None
                }
            },
            public_key: {
                if let Some(o) = builder.public_key() {
                    Some(o.clone())
                } else {
                    None
                }
            },
            content: builder.desc().clone(),
        }
    }

    pub fn object_type_code(&self) -> ObjectTypeCode {
        self.content.object_type_code()
    }

    pub fn create_timestamp(&self) -> u64 {
        self.create_timestamp
    }

    pub fn expired_time(&self) -> Option<u64> {
        self.expired_time
    }

    pub fn owner(&self) -> Option<&T::OwnerObj> {
        self.owner.as_ref()
    }

    pub fn area(&self) -> Option<&T::AreaObj> {
        self.area.as_ref()
    }

    pub fn author(&self) -> Option<&T::AuthorObj> {
        self.author.as_ref()
    }

    pub fn public_key(&self) -> Option<&T::PublicKeyObj> {
        self.public_key.as_ref()
    }

    pub fn content(&self) -> &T {
        &self.content
    }

    pub fn mut_content(&mut self) -> &mut T {
        &mut self.content
    }
}

impl<T: ObjectDescTrait + Serialize + Deserialize + std::default::Default> Serialize for NamedObjectDesc<T> {
    fn raw_capacity(&self) -> usize {
        self.create_timestamp.raw_capacity() +
        self.expired_time.raw_capacity() +
        self.owner.raw_capacity() +
        self.area.raw_capacity() +
        self.author.raw_capacity() + 
        self.public_key.raw_capacity() + 
        self.content.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.create_timestamp.serialize(buf)?;
        let buf = self.expired_time.serialize(buf)?;
        let buf = self.owner.serialize(buf)?;
        let buf = self.area.serialize(buf)?;
        let buf = self.author.serialize(buf)?;
        let buf = self.public_key.serialize(buf)?;
        let buf = self.content.serialize(buf)?;

        Ok(buf)
    }

}

impl<T: ObjectDescTrait + Serialize + Deserialize + std::default::Default> Deserialize for NamedObjectDesc<T> {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (create_timestamp, buf) = u64::deserialize(buf)?;
        let (expired_time, buf) = Option::<u64>::deserialize(buf)?;
        let (owner, buf) = Option::<T::OwnerObj>::deserialize(buf)?;
        let (area, buf) = Option::<T::AreaObj>::deserialize(buf)?;
        let (author, buf) = Option::<T::AuthorObj>::deserialize(buf)?;
        let (public_key, buf) = Option::<T::PublicKeyObj>::deserialize(buf)?;
        let (content, buf) = T::deserialize(buf)?;

        Ok((Self{
            create_timestamp: create_timestamp,
            expired_time: expired_time,
            owner: owner,
            area: area,
            author: author,
            public_key: public_key,
            content,
        }, buf))
    }

}

/// named
/// object body content
#[derive(Clone, Default)]
pub struct NamedObjectBody<T: ObjectBodyTrait + Serialize + Deserialize + std::default::Default> {
    update_time: u64,
    content: T,
    user_data: Option<Vec<u8>>,
}

impl<T: ObjectBodyTrait + Serialize + Deserialize + std::default::Default> std::fmt::Display for NamedObjectBody<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "update_time: [{}], user_data: [{:?}], content: [{}]",
               self.update_time,
               self.user_data,
               self.content,
            )
    }
}

impl<T: ObjectBodyTrait + Serialize + Deserialize + std::default::Default> NamedObjectBody<T> {
    pub fn build(builder: &ObjectBodyBuilder<T>)  -> NamedObjectBody<T> {
        Self{
            update_time: now(),
            content: builder.body().clone(),
            user_data: None,
        }
    }

    pub fn update_time(&self) -> u64 {
        self.update_time
    }

    pub fn user_data(&self) -> Option<&[u8]> {
        self.user_data
            .as_ref()
            .map(| data | data.as_slice() )
    }

    pub fn content(&self) -> &T {
        &self.content
    }

    pub fn set_update_time(&mut self) {
        self.update_time = now();
    }

    pub fn mut_content(&mut self) -> &mut T {
        &mut self.content
    }

    pub fn set_user_data(&mut self, data: Option<Vec<u8>>) {
        self.user_data = data;
    }
}

impl<T: ObjectBodyTrait + Serialize + Deserialize + std::default::Default> Serialize for NamedObjectBody<T> {
    fn raw_capacity(&self) -> usize {
        self.update_time.raw_capacity() + 
        self.content.raw_capacity() + 
        self.user_data.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.update_time.serialize(buf)?;
        let buf = self.content.serialize(buf)?;
        let buf = self.user_data.serialize(buf)?;

        Ok(buf)
    }
}

impl<T: ObjectBodyTrait + Serialize + Deserialize + std::default::Default> Deserialize for NamedObjectBody<T> {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (update_time, buf) = u64::deserialize(buf)?;
        let (content, buf) = T::deserialize(buf)?;
        let (user_data, buf) = Option::<Vec<u8>>::deserialize(buf)?;

        Ok((Self{
            update_time, content, user_data,
        }, buf))
    }
}


/// named
/// object all content
#[derive(Clone, Default)]
pub struct NamedObject<DESC, BODY>
where DESC: ObjectDescTrait + Serialize + Deserialize + std::default::Default,
      BODY: ObjectBodyTrait + Serialize + Deserialize + std::default::Default {
    object_id: ObjectId,
    desc: NamedObjectDesc<DESC>,
    body: NamedObjectBody<BODY>,
    nonce: Option<String>,
}

impl<DESC, BODY> std::fmt::Debug for NamedObject<DESC, BODY> 
where DESC: ObjectDescTrait + Serialize + Deserialize + std::default::Default,
      BODY: ObjectBodyTrait + Serialize + Deserialize + std::default::Default {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Display).fmt(f)
    }
}

impl<DESC, BODY> std::fmt::Display for NamedObject<DESC, BODY> 
where DESC: ObjectDescTrait + Serialize + Deserialize + std::default::Default,
      BODY: ObjectBodyTrait + Serialize + Deserialize + std::default::Default {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
               r#"object-id: [{}], desc: [{}], body: [{}], nonce: [{:?}]"#, 
               self.object_id, 
               self.desc,
               self.body,
               self.nonce)
    }
}

impl<DESC, BODY> NamedObject<DESC, BODY>
where DESC: ObjectDescTrait + Serialize + Deserialize + std::default::Default,
      BODY: ObjectBodyTrait + Serialize + Deserialize + std::default::Default {
    pub fn build(id: ObjectId, 
                 desc: NamedObjectDesc<DESC>, 
                 body: NamedObjectBody<BODY>) -> Self {
        Self {
            object_id: id, desc, body, nonce: None
        }
    }

    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    pub fn desc(&self) -> &NamedObjectDesc<DESC> {
        &self.desc
    }

    pub fn mut_desc(&mut self) -> &mut NamedObjectDesc<DESC> {
        &mut self.desc
    }

    pub fn body(&self) -> &NamedObjectBody<BODY> {
        &self.body
    }

    pub fn mut_body(&mut self) -> &mut NamedObjectBody<BODY> {
        &mut self.body
    }

    pub fn set_nonce(&mut self, nonce: Option<String>) {
        self.nonce = nonce;
    }

}

impl<DESC, BODY> Serialize for NamedObject<DESC, BODY>
where DESC: ObjectDescTrait + Serialize + Deserialize + std::default::Default,
      BODY: ObjectBodyTrait + Serialize + Deserialize + std::default::Default {
    fn raw_capacity(&self) -> usize {
        self.object_id.raw_capacity() + 
        self.desc.raw_capacity() + 
        self.body.raw_capacity() + 
        self.nonce.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.object_id.serialize(buf)?;
        let buf = self.desc.serialize(buf)?;
        let buf = self.body.serialize(buf)?;
        let buf = self.nonce.serialize(buf)?;

        // TODO
        // 加上签名
        Ok(buf)

    }
}

impl<DESC, BODY> Deserialize for NamedObject<DESC, BODY>
where DESC: ObjectDescTrait + Serialize + Deserialize + std::default::Default,
      BODY: ObjectBodyTrait + Serialize + Deserialize + std::default::Default {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (object_id, buf) = ObjectId::deserialize(buf)?;
        let (desc_content, buf) = NamedObjectDesc::<DESC>::deserialize(buf)?;
        let (body_content, buf) = NamedObjectBody::<BODY>::deserialize(buf)?;
        let (nonce, buf) = Option::<String>::deserialize(buf)?;

        Ok((Self{
            object_id, desc: desc_content, body: body_content, nonce
        }, buf))

    }
}


