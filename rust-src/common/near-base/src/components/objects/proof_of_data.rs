
use crate::{components::ObjectTypeCode, errors::*, public_key::PublicKey, Area, Deserialize, ObjectBodyTrait, ObjectDescTrait, Serialize};

use super::{object_type::ObjectId,
            object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody},
            OtherObjectSubCode,
            /* general::EndpointExt */};

pub type ProofOfDataDesc<T> = NamedObjectDesc<ProofOfDataDescContent<T>>;
pub type ProofOfDataBody<C> = NamedObjectBody<ProofOfDataBodyContent<C>>;
pub type ProofOfDataObject<T, C> = NamedObject<ProofOfDataDescContent<T>, ProofOfDataBodyContent<C>>;

#[derive(Clone)]
pub struct ProofOfDataDescContent<T> {
    object_type_code: ObjectTypeCode,
    proof_data: T,
}

impl<T: std::default::Default> std::default::Default for ProofOfDataDescContent<T> {
    fn default() -> Self {
        Self {
            object_type_code: ObjectTypeCode::Other(OtherObjectSubCode::OBJECT_TYPE_OTHER_PROOFDATA as u8),
            proof_data: Default::default(),
        }
    }
}

impl<T: std::default::Default> ProofOfDataDescContent<T> {

    pub fn set_proof_data(&mut self, proof_data: T) {
        let _ = std::mem::replace(&mut self.proof_data, proof_data);
    }

    pub fn take_proof_data(&mut self) -> T {
        std::mem::replace(&mut self.proof_data, T::default())
    }

    pub fn proof_data(&self) -> &T {
        &self.proof_data
    }

}

impl<T> std::fmt::Display for ProofOfDataDescContent<T> 
where T: Clone + std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "object_type_code: [{}], proof_data: [{}]", self.object_type_code, self.proof_data)
    }
}

impl<T> ObjectDescTrait for ProofOfDataDescContent<T>
where T: Clone + std::fmt::Display {
    fn object_type_code(&self) -> ObjectTypeCode {
        self.object_type_code
    }

    type OwnerObj = ObjectId;
    type AreaObj = Area;
    type AuthorObj = ObjectId;
    type PublicKeyObj = PublicKey;

}

impl<T> Serialize for ProofOfDataDescContent<T>
where T: Clone + std::fmt::Display + Serialize {
    fn raw_capacity(&self) -> usize {
        self.object_type_code.raw_capacity() + 
        self.proof_data.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.object_type_code.serialize(buf)?;
        let buf = self.proof_data.serialize(buf)?;

        Ok(buf)
    }

}

impl<T> Deserialize for ProofOfDataDescContent<T>
where T: Clone + std::fmt::Display + Serialize + Deserialize {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (object_type_code, buf) = ObjectTypeCode::deserialize(buf)?;
        let (proof_data, buf) = T::deserialize(buf)?;

        Ok((Self{
            object_type_code, proof_data, 
        }, buf))
    }

}

#[derive(Clone)]
pub struct ProofOfDataBodyContent<C> {
    core_data: C,
}

impl<C> std::default::Default for ProofOfDataBodyContent<C> 
where C: std::default::Default {
    fn default() -> Self {
        Self {
            core_data: Default::default()
        }
    }
}

impl<C: std::default::Default> ProofOfDataBodyContent<C> {
    pub fn set_data(&mut self, data: C) {
        self.core_data = data;
    }

    pub fn take_data(&mut self) -> C {
        std::mem::replace(&mut self.core_data, C::default())
    }

    pub fn data(&self) -> &C {
        &self.core_data
    }

}

impl<C> std::fmt::Display for ProofOfDataBodyContent<C> 
where C: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "core_data: [{}]", self.core_data)
    }
}

impl<C> ObjectBodyTrait for ProofOfDataBodyContent<C>
where C: Clone + std::fmt::Display { }

impl<C> Serialize for ProofOfDataBodyContent<C> 
where C: Serialize {
    fn raw_capacity(&self) -> usize {
        self.core_data.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.core_data.serialize(buf)?;

        Ok(buf)
    }

}

impl<C> Deserialize for ProofOfDataBodyContent<C> 
where C: Deserialize {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (core_data, buf) = C::deserialize(buf)?;

        Ok((Self{
            core_data
        }, buf))
    }

}

