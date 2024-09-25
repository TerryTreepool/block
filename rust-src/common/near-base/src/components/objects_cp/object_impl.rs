
use crate::{Serialize, Deserialize};
use crate::errors::*;

use super::{object_type::{ObjectId, ObjectTypeCode},
            object_builder::{ObjectDescBuilder, ObjectBodyBuilder, 
                             ObjectDescTrait, ObjectBodyTrait}};

// /// AreaObjTrait
// #[derive(Clone)]
// pub struct AreaObjWrapper {
//     pub area: Area,
// }

// impl AreaObjTrait for AreaObjWrapper {
//     fn into_area(&self) -> Area {
//         self.area.clone()
//     }
// }

// impl Serialize for AreaObjWrapper {
//     fn raw_capacity(&self) -> usize {
//         self.area.raw_capacity()
//     }

//     fn serialize<'a>(&self,
//                      buf: &'a mut [u8],
//                      builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
//         self.area.serialize(buf)
//     }
// }

// impl Deserialize for AreaObjWrapper {
//     fn deserialize<'de>(buf: &'de [u8], 
//                         builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
//         let (area, buf) = Area::deserialize(buf)?;
//         Ok((Self{
//             area
//         }, buf))
//     }

// }

/// named
/// object desc content
#[derive(Clone)]
pub struct NamedObjectDesc<T: ObjectDescTrait + Serialize + Deserialize> {
    create_timestamp: u64,
    expired_time: Option<u64>,

    owner: Option<T::OwnerObj>,
    area: Option<T::AreaObj>,
    author: Option<T::AuthorObj>,
    public_key: Option<T::PublicKeyObj>,

    desc: T,
}

impl<T: ObjectDescTrait + Serialize + Deserialize> NamedObjectDesc<T> {
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
            desc: builder.desc().clone(),
        }
    }

    pub fn object_type_code(&self) -> ObjectTypeCode {
        self.desc.object_type_code()
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
}

impl<T: ObjectDescTrait + Serialize + Deserialize> Serialize for NamedObjectDesc<T> {
    fn raw_capacity(&self) -> usize {
        0   // auto size
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.create_timestamp.serialize(buf)?;

        let buf = self.expired_time.serialize(buf)?;

        let buf = self.owner.serialize(buf)?;

        let buf = self.area.serialize(buf)?;

        let buf = self.author.serialize(buf)?;

        let buf = self.public_key.serialize(buf)?;

        self.desc.serialize(buf)
    }

}

impl<T: ObjectDescTrait + Serialize + Deserialize> Deserialize for NamedObjectDesc<T> {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (create_timestamp, buf) = u64::deserialize(buf)?;

        let (expired_time, buf) = Option::<u64>::deserialize(buf)?;

        let (owner, buf) = Option::<T::OwnerObj>::deserialize(buf)?;

        let (area, buf) = Option::<T::AreaObj>::deserialize(buf)?;

        let (author, buf) = Option::<T::AuthorObj>::deserialize(buf)?;

        let (public_key, buf) = Option::<T::PublicKeyObj>::deserialize(buf)?;

        let (desc, buf) = T::deserialize(buf)?;

        Ok((Self{
            create_timestamp: create_timestamp,
            expired_time: expired_time,
            owner: owner,
            area: area,
            author: author,
            public_key: public_key,
            desc,
        }, buf))
    }

}

/// named
/// object body content
#[derive(Clone)]
pub struct NamedObjectBody<T: ObjectBodyTrait + Serialize + Deserialize> {
    body: T
}

impl<T: ObjectBodyTrait + Serialize + Deserialize> NamedObjectBody<T> {
    pub fn build(builder: &ObjectBodyBuilder<T>)  -> NamedObjectBody<T> {
        Self{
            body: builder.body().clone()
        }
    }
}

impl<T: ObjectBodyTrait + Serialize + Deserialize> Serialize for NamedObjectBody<T> {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.body.serialize(buf)
    }
}

impl<T: ObjectBodyTrait + Serialize + Deserialize> Deserialize for NamedObjectBody<T> {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (body, buf) = T::deserialize(buf)?;
        Ok((Self{body}, buf))
    }
}


/// named
/// object all content
#[derive(Clone)]
pub struct NamedObject<DESC, BODY>
where DESC: ObjectDescTrait + Serialize + Deserialize,
      BODY: ObjectBodyTrait + Serialize + Deserialize {
    object_id: ObjectId,
    desc: NamedObjectDesc<DESC>,
    body: NamedObjectBody<BODY>,
    nonce: Option<String>,
}

impl<DESC, BODY> NamedObject<DESC, BODY>
where DESC: ObjectDescTrait + Serialize + Deserialize,
      BODY: ObjectBodyTrait + Serialize + Deserialize {
    pub fn build(id: ObjectId, 
                 desc: NamedObjectDesc<DESC>, 
                 body: NamedObjectBody<BODY>) -> Self {
        Self {
            object_id: id, desc, body, nonce: None
        }
    }

    // pub fn id(&self) -> &ObjectId {
    //     &self.object_id
    // }

    pub fn desc(&self) -> &NamedObjectDesc<DESC> {
        &self.desc
    }

    pub fn body(&self) -> &NamedObjectBody<BODY> {
        &self.body
    }

    pub fn mut_body(&mut self) -> &mut NamedObjectBody<BODY> {
        &mut self.body
    }

    pub fn set_nonce(&mut self, nonce: String) {
        self.nonce = Some(nonce);
    }

    pub fn clear_nonce(&mut self) {
        self.nonce = None;
    }

}

impl<DESC, BODY> Serialize for NamedObject<DESC, BODY>
where DESC: ObjectDescTrait + Serialize + Deserialize,
      BODY: ObjectBodyTrait + Serialize + Deserialize {
    fn raw_capacity(&self) -> usize {
        0
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
where DESC: ObjectDescTrait + Serialize + Deserialize,
      BODY: ObjectBodyTrait + Serialize + Deserialize {
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


