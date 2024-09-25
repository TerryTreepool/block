
use crate::{Serialize, Deserialize};

// pub trait ObjectTrait: Serialize + Deserialize + Clone {
//     fn id(&self) -> &ObjectId;
//     fn set_nonce(&mut self, nonce: Option<String>);

// }

mod object_impl;

pub mod object_type;
pub mod object_builder;
pub mod service;
pub mod device;
pub mod thing;
// pub mod general;

pub use object_type::{ObjectId, ObjectTypeCode};
pub use object_builder::{ObjectDescTrait, ObjectDescBuilder,
                         ObjectBodyTrait, ObjectBodyBuilder,
                         ObjectBuilder};
pub use object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody};
pub use device::{DeviceDesc, DeviceBody, DeviceObject};
pub use service::{ServiceDesc, ServiceBody, ServiceObject};
pub use thing::{ThingDesc, ThingBody, ThingObject};
// pub use general::*;

pub mod object_component {
    use std::{sync::Arc, any::Any, boxed::Box, convert::TryInto};

    use rand::distributions::uniform::UniformInt;

    use crate::{Area, crypto_module::PublicKey, NearError, ObjectBodyTrait};

    use super::{object_type::{ObjectId, ObjectTypeCode}, object_builder::{ObjectDescTrait}};
    use super::{ServiceObject, ServiceDesc, ServiceBody};
    use super::{DeviceObject, DeviceDesc, DeviceBody};
    use super::{ThingObject, ThingDesc, ThingBody};

    macro_rules! downcast_component {
        ($dynamic_object: expr, $handler: expr) => {
            match $dynamic_object.object_type_code() {
                ObjectTypeCode::Service(_) => $handler($dynamic_object.as_any().downcast_ref::<ServiceObject>().unwrap()),
                // ObjectTypeCode::Device(_) => $handler($dynamic_object.as_any().downcast_ref::<DeviceObject>().unwrap()),
                // ObjectTypeCode::Thing(_) => $handler($dynamic_object.as_any().downcast_ref::<ThingObject>().unwrap()),
                _ => { unimplemented!() }
            }
        };
        ($dynamic_object: expr) => {
            downcast_component!($dynamic_object, |p| p)
        };
    }

    // #[derive(Clone)]
    enum Type {
        // // o: Box<dyn Any + Send + Sync>,
        // // Service(ServiceObject),
        // /// 设备属性，如黑盒
        // // Device(DeviceObject),
        // // /// 用户属性，如玩家，玩家组等
        // // People(PeopleObject),
        // // /// Thing属性，如灯等
        // // Thing(ThingObject),
        // // /// 其他属性
        // // Other(OtherObject),
    }


    // #[derive(Clone)]
    struct ComponentImpl {
        object_type_code: ObjectTypeCode,
        object: Box<dyn Any + Send + Sync>,
    }

    struct ComponentContext<'a> {
        id: &'a ObjectId,
        nonce: &'a Option<String>,

        create_timestamp: u64,
        expired_time: Option<u64>,
        owner: Option<T::OwnerObj>,
        area: Option<T::AreaObj>,
        author: Option<T::AuthorObj>,
        public_key: Option<T::PublicKeyObj>,
    

    }

    #[derive(Clone)]
    pub struct Component(Arc<ComponentImpl>);

    impl From<ServiceObject> for Component {
        fn from(o: ServiceObject) -> Self {
            Self(Arc::new(ComponentImpl{
                object_type_code: o.desc().object_type_code(),
                object: Box::new(o)
            }))
        }
    }

    impl From<DeviceObject> for Component {
        fn from(o: DeviceObject) -> Self {
            Self(Arc::new(ComponentImpl{
                object_type_code: o.desc().object_type_code(),
                object: Box::new(o)
            }))
        }
    }

    impl From<ThingObject> for Component {
        fn from(o: ThingObject) -> Self {
            Self(Arc::new(ComponentImpl{
                object_type_code: o.desc().object_type_code(),
                object: Box::new(o)
            }))
        }
    }

    impl AsRef<ServiceObject> for Component {
        fn as_ref(&self) -> &ServiceObject {
            match self.0.object_type_code {
                ObjectTypeCode::Service(_) => self.0.object.downcast_ref::<ServiceObject>().unwrap(),
                _ => unimplemented!()
            }
        }
    }

    impl AsRef<DeviceObject> for Component {
        fn as_ref(&self) -> &DeviceObject {
            match self.0.object_type_code {
                ObjectTypeCode::Device(_) => self.0.object.downcast_ref::<DeviceObject>().unwrap(),
                _ => unimplemented!()
            }
        }
    }

    impl AsRef<ThingObject> for Component {
        fn as_ref(&self) -> &ThingObject {
            match self.0.object_type_code {
                ObjectTypeCode::Thing(_) => self.0.object.downcast_ref::<ThingObject>().unwrap(),
                _ => unimplemented!()
            }
        }
    }

    impl Component {
        pub fn object_type_code(&self) -> &ObjectTypeCode {
            &self.0.object_type_code
        }

        // pub fn as_any<'a>(&'a self) -> &'a dyn Any {
        //     self.0.object.as_ref()
        // }
    }

    // impl<T: 'static + Send + Sync> TryInto<T> for Component {
    //     type Error = NearError;

    //     fn try_into(self) -> Result<T, Self::Error> {
    //         match &self.0.code {
    //             ObjectTypeCode::Service(_) => Ok(self.0.o.downcast_ref::<ServiceObject>().unwrap().clone()),
    //             // ObjectTypeCode::Device(_) => $handler($dynamic_object.as_any().downcast_ref::<DeviceObject>().unwrap()),
    //             // ObjectTypeCode::Thing(_) => $handler($dynamic_object.as_any().downcast_ref::<ThingObject>().unwrap()),
    //             _ => unimplemented()
    //             }
    //     }
    
    // }

    impl Component {
        // pub fn id(&self) -> &ObjectId {
        //     let id = {
        //         match self.object_type_code() {
        //             ObjectTypeCode::Service(_) => self.as_any().downcast_ref::<ServiceObject>().unwrap().id(),
        //             ObjectTypeCode::Device(_) => self.as_any().downcast_ref::<DeviceObject>().unwrap().id(),
        //             ObjectTypeCode::Thing(_) => self.as_any().downcast_ref::<ThingObject>().unwrap().id(),
        //             _ => unimplemented!()
        //         }
        //     };
        //     // let r = downcast_component!(self, | component: &ServiceObject | component.id() );
        //     id
        // }
        // pub fn id(&self) -> &ObjectId {
        //     match self.0.code {
        //         ObjectTypeCode::Service(_) => {
        //             let r = self.0.o.downcast_ref::<ServiceObject>();
        //         }
        //         ObjectTypeCode::Device(_) => {
        //             let r = self.0.o.downcast_ref::<ServiceObject>();
        //         }
        //         // ObjectTypeCode::People(_) => {
        //         //     let r = self.0.o.downcast_ref::<ServiceObject>();
        //         // }
        //         ObjectTypeCode::Thing(_) => {
        //             let r = self.0.o.downcast_ref::<ServiceObject>();
        //         }
        //         // ObjectTypeCode::Other(_) => {
        //         //     let r = self.0.o.downcast_ref::<ServiceObject>();
        //         // }    /// 设备属性，如黑盒
        //         _ => unimplemented!()
        //     }
        // }
    }
    

    // impl From<DeviceObject> for Component {
    //     fn from(o: DeviceObject) -> Self {
    //         Self(Arc::new(ComponentImpl{
    //             o: Type::Device(o)
    //         }))
    //     }
    // }

    // impl Component {
    //     pub fn id(&self) -> &ObjectId {
    //         match &self.0.o {
    //             Type::Service(o) => o.id(),
    //             Type::Device(o) => o.id(),
    //             // Type::Thing(o) => o.id(),
    //             _ => { unreachable!() }
    //         }
    //         // &self.object_id
    //     }

    //     pub fn create_timestamp(&self) -> u64 {
    //         match &self.0.o {
    //             Type::Service(o) => o.desc().create_timestamp(),
    //             Type::Device(o) => o.desc().create_timestamp(),
    //             // Type::Thing(o) => o.id(),
    //             _ => { unreachable!() }
    //         }
    //     }

    //     pub fn expired_time(&self) -> Option<u64> {
    //         match &self.0.o {
    //             Type::Service(o) => o.desc().expired_time(),
    //             Type::Device(o) => o.desc().expired_time(),
    //             // Type::Thing(o) => o.id(),
    //             _ => { unreachable!() }
    //         }
    //     }
    
    // }

    // impl ObjectDescTrait for Component {
    //     fn object_type_code(&self) -> ObjectTypeCode {
    //         match &self.0.o {
    //             Type::Service(o) => o.desc().object_type_code(),
    //             Type::Device(o) => o.desc().object_type_code(),
    //             // Type::Thing(o) => o.id(),
    //             _ => { ObjectTypeCode::Unknown }
    //         }
    //     }

    //     fn version(&self) -> u8 {
    //         match &self.0.o {
    //             Type::Service(o) => o.desc().desc().version(),
    //             Type::Device(o) => o.desc().desc().version(),
    //             // Type::Thing(o) => o.id(),
    //             _ => { unreachable!() }
    //         }
    //     }

    //     type OwnerObj = ObjectId;
    //     type AreaObj = Area;
    //     type AuthorObj = ObjectId;
    //     type PublicKeyObj = PublicKey;
    // }

    // impl AsRef<ServiceDesc> for Component {
    //     fn as_ref(&self) -> &ServiceDesc {
    //         match &self.o {
    //             Type::Service(o) => o.desc().desc(),
    //             Type::Device(o) => o.id(),
    //             // Type::Thing(o) => o.id(),
    //             _ => { unreachable!() }
    //         }
    //     }

    // }
        // pub fn owner(&self) -> Option<T::OwnerObj> {
        //     match &self.o {
        //         Type::Service(o) => o.desc().expired_time(),
        //         Type::Device(o) => o.desc().expired_time(),
        //         // Type::Thing(o) => o.id(),
        //         _ => { unreachable!() }
        //     }
        // }

        // owner: Option<T::OwnerObj>,
        // area: Option<T::AreaObj>,
        // author: Option<T::AuthorObj>,
        // public_key: Option<T::PublicKeyObj>,

        // desc: T,
        // // pub fn as_desc(&self) -> &NamedObjectDesc<DESC> {
        //     &self.desc
        // }

        // pub fn body(&self) -> &NamedObjectBody<BODY> {
        //     &self.body
        // }
    // }



}
