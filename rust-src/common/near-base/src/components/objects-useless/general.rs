
use std::any::Any;

use crate::{components::{ObjectTypeCode}, 
            Serialize, Deserialize,
            public_key::PublicKey,
            errors::*, ObjectBodyTrait, ObjectDescTrait,
            Area, ServiceDesc};

use super::{object_type::{ObjectId},
            object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody}};
use super::{service::{ServiceObject, ServiceDescContent}, DeviceObject, ThingObject};

#[derive(Clone)]
pub struct DynamicObject {
    object_code: ObjectTypeCode,
    object: Box<dyn Any + Send + Sync>,
}

// impl DynamicObject {
//     pub fn into_desc(&self) -> Box<&dyn ObjectDescTrait> {
//         match self.object_code {
//             ObjectTypeCode::Service(_) => {
//                 let desc = self.object.downcast_ref::<ServiceObject>().unwrap().desc();
//                 type service_owner = ObjectDescTrait<OwnerObj=ServiceDesc
//             }
//             // ObjectTypeCode::Device(_) => self.object.downcast_ref::<DeviceObject>().unwrap().desc(),
//             // ObjectTypeCode::People(_) => { unimplemented!() },
//             // ObjectTypeCode::Thing(_) => self.object.downcast_ref::<ThingObject>().unwrap().desc(),
//             // ObjectTypeCode::Other(_) => { unimplemented!() },
//             _ => { unreachable!() }
//         }
//     }
// }

impl ObjectDescTrait for DynamicObject {
    fn object_type_code(&self) -> ObjectTypeCode {
        self.object_code
    }

    type OwnerObj = <ServiceDescContent as ObjectDescTrait>::OwnerObj;
    type AreaObj = <ServiceDescContent as ObjectDescTrait>::AreaObj;
    type AuthorObj = <ServiceDescContent as ObjectDescTrait>::AuthorObj;
    type PublicKeyObj = <ServiceDescContent as ObjectDescTrait>::PublicKeyObj;

}

impl<T> AsRef<T> for DynamicObject
where T: 'static {
    fn as_ref(&self) -> &T {
        self.object.downcast_ref::<T>().unwrap()
    }
}

impl From<ServiceObject> for DynamicObject {
    fn from(value: ServiceObject) -> Self {
        Self {
            object_code: value.desc().object_type_code(),
            object: Box::new(value)
        }
    }
}
// use crate::{endpoints::Endpoint,
//             ObjectId, Serialize,
//             errors::*, Deserialize};

// #[derive(Clone)]
// pub struct EndpointExt {
//     endpoints: Vec<Endpoint>,
//     stun_node_list: Vec<ObjectId>,
//     turn_node_list: Vec<ObjectId>,
// }

// impl std::default::Default for EndpointExt {
//     fn default() -> Self {
//         Self {
//             endpoints: Vec::new(),
//             stun_node_list: Vec::new(),
//             turn_node_list: Vec::new(),
//         }
//     }
// }

// impl Serialize for EndpointExt {
//     fn raw_capacity(&self) -> usize {
//         0
//     }

//     fn serialize<'a>(&self,
//                      buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
//         let buf = self.endpoints.serialize(buf)?;

//         let buf = self.stun_node_list.serialize(buf)?;

//         let buf = self.turn_node_list.serialize(buf)?;

//         Ok(buf)

//     }

// }

// impl Deserialize for EndpointExt {
//     fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
//         let (endpoints, buf) = Vec::<Endpoint>::deserialize(buf)?;

//         let (stun_node_list, buf) = Vec::<ObjectId>::deserialize(buf)?;

//         let (turn_node_list, buf) = Vec::<ObjectId>::deserialize(buf)?;

//         Ok((Self {
//             endpoints, stun_node_list, turn_node_list
//         }, buf))
//     }

// }
