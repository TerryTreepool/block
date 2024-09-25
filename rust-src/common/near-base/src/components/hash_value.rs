
use base58::ToBase58;
use generic_array::typenum::{marker_traits::Unsigned, U32};
use generic_array::GenericArray;

use crate::ObjectId;
use crate::codec::{Serialize, Deserialize};
use crate::errors::*;

// #[derive(Clone)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct Hash256(GenericArray<u8, U32>);

impl Hash256 {
    pub fn len() -> usize {
        U32::to_usize()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }

    pub fn to_hex_string(&self) -> String {
        hex::encode_upper(self.0.as_slice())
    }

}

impl Into<ObjectId> for Hash256 {
    fn into(self) -> ObjectId {
        ObjectId::from(self.0)
    }
}

impl AsRef<GenericArray<u8, U32>> for Hash256 {
    fn as_ref(&self) -> &GenericArray<u8, U32> {
        &self.0
    }
}

impl std::default::Default for Hash256 {
    fn default() -> Self {
        Self(GenericArray::default())
    }
}

impl From<&[u8; 32]> for Hash256 {
    fn from(buf: &[u8; 32]) -> Self {
        Self(GenericArray::clone_from_slice(buf))
    }
}

impl From<&[u8]> for Hash256 {
    fn from(buf: &[u8]) -> Self {
        let mut ret = Self::default();
        ret.0.as_mut_slice().copy_from_slice(&buf);
        ret
    }
}

impl From<Hash256> for GenericArray<u8, U32> {
    fn from(v: Hash256) -> Self {
        v.0
    }
}

impl From<GenericArray<u8, U32>> for Hash256 {
    fn from(hash: GenericArray<u8, U32>) -> Self {
        Self(hash)
    }
}

// impl std::cmp::PartialEq<Hash256> for Hash256 {
//     fn eq(&self, other: &Hash256) -> bool {
//         self.0 == other.0
//     }

// }

impl std::fmt::Display for Hash256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_slice().to_base58())
    }

}

impl Serialize for Hash256 {
    fn raw_capacity(&self) -> usize {
        self.0.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.0.serialize(buf)
    }

}

impl Deserialize for Hash256 {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (val, buf) = GenericArray::<u8, U32>::deserialize(buf)?;

        Ok((Self(val), buf))
    }

}

#[cfg(test)]
mod test {
    // use crypto::{sha1::Sha1, digest::Digest};

    use crate::{Deserialize, Hash256, Serialize, hash_data};


    #[test]
    fn test() {
        {
            let mut a = Box::new(vec![0,1,2,3,4,5,6,7]);
            // let mut b = Box::new(vec![1]);
            let mut b = vec![1];

            std::mem::swap(a.as_mut(), b.as_mut());

            println!("{:?}", a);
            println!("/////////////////////////////////////////");
        }
        // {
        //     let mut hasher = Sha1::new();
        //     let mut outer = vec![0u8; 128];

        //     let v = [0u8, 1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,0,214,134,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123];

        //     // let text = String::from("123456");
        
        //     hasher.input(&v);
        //     hasher.result(outer.as_mut());

        //     println!("{:?}", outer);

        //     println!("=> {}", hasher.result_str());
        //     // let mut hasher = DefaultHasher::new();
        //     // let v = [0u8, 1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,0,214,134,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123];
        //     // v.hash_slice(&mut hasher);
        //     // Hash::hash_slice(&v, &mut hasher);
        //     // let h = hasher.finish();
        //     // println!("`{}` hash is {}, hash bytes is {:?}", s, h, h.to_be_bytes());
        // }

        {
            let v = [0u8, 1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,0,214,134,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123,1,23,34,1,3,53,23,32,1,54,214,34,123];
            // let h = Hash256::from(&v[0..32]);
            let h = hash_data(&v[..]);

            println!("hash256 len={}", Hash256::len());
            println!("h={}", h);

            let mut b = [0u8; 1024];

            let _ = h.serialize(&mut b);

            {
                let (h_p, _) = crate::Hash256::deserialize(&b).unwrap();
                println!("h_p={}", h_p);

            }
        }

    }
}
