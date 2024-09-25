
use crate::{components::{Endpoint, Area}, 
            {Serialize, Deserialize},
            errors::*, public_key::PublicKey};

use super::{object_type::{ObjectId, ObjectTypeCode},
            object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody},
            object_builder::{ObjectDescTrait, ObjectBodyTrait}};


pub type ServiceDesc = NamedObjectDesc<ServiceDescContent>;
pub type ServiceBody = NamedObjectBody<ServiceBodyContent>;
pub type ServiceObject = NamedObject<ServiceDescContent, ServiceBodyContent>;

#[derive(Clone)]
pub struct ServiceDescContent {
    object_type_code: ObjectTypeCode,
    id: ObjectId
}

impl ServiceDescContent {
    pub fn new(code: u8) -> Self {
        Self{
            object_type_code: ObjectTypeCode::Service(code),
            id: ObjectId::default(),
        }
    }

}

impl Serialize for ServiceDescContent {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.object_type_code.serialize(buf)?;

        let buf = self.id.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for ServiceDescContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (object_type_code, buf) = ObjectTypeCode::deserialize(buf)?;

        let (id, buf) = ObjectId::deserialize(buf)?;

        Ok((Self{
            object_type_code, id
        }, buf))
    }

}

impl ObjectDescTrait for ServiceDescContent {
    fn object_type_code(&self) -> ObjectTypeCode {
        self.object_type_code
    }

    type OwnerObj = ObjectId;
    type AreaObj = Area;
    type AuthorObj = ObjectId;
    type PublicKeyObj = PublicKey;

}

#[derive(Clone)]
pub struct ServiceBodyContent {
    endpoints: Vec<Endpoint>,
    name: Option<String>
}

impl ServiceBodyContent {
    pub fn new(endpoints: Vec<Endpoint>, name: Option<String>) -> Self {
        Self{
            endpoints, name
        }
    }
}

impl ObjectBodyTrait for ServiceBodyContent {}

impl std::default::Default for ServiceBodyContent {
    fn default() -> Self {
        Self {
            endpoints: vec![],
            name: None
        }
    }
}

impl Serialize for ServiceBodyContent {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.endpoints.serialize(buf)?;

        let buf = self.name.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for ServiceBodyContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (endpoints, buf) = Vec::<Endpoint>::deserialize(buf)?;

        let (name, buf) = Option::<String>::deserialize(buf)?;

        Ok((Self{
            endpoints: endpoints,
            name: name,
        }, buf))
    }

}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::{Area, Endpoint,
                codec::*, service::test};

    use super::{ServiceObject, ServiceDescContent, ServiceBodyContent};
    use crate::ObjectBuilder;

    #[test]
    fn test_object() {
        let service = 
            ObjectBuilder::new(ServiceDescContent::new(10), 
                               ServiceBodyContent::new(vec![Endpoint::from_str("D4Tcp0.0.0.0:19220").unwrap()], None))
                .update_desc(|desc| {
                    desc.set_area(Area::new(123, 10, 2321, 10));
                })
                .update_body(|_body| {
                })
                .build().unwrap();

        println!("service-id={}", service.id().to_string());
        let mut b = [0u8;1024];

        {
            let _ = service.serialize(&mut b).unwrap();

            println!("{:?}", b);
        }

        {
            let (r, _) = ServiceObject::deserialize(&b).unwrap();

            println!("service-id={}", r.id().to_string());

        }
    }
}


