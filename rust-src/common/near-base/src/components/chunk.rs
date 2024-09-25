
use generic_array::{GenericArray, typenum::{U32, }};
use base58::ToBase58;

use crate::{Hash256, Serialize, Deserialize, RawFixedBytes, CheckSum, ObjectId, };

pub const CHUNK_MAX_LEN: u64 = 1024 * 1024;

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct ChunkId(GenericArray<u8, U32>);

impl std::fmt::Debug for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Display).fmt(f)
    }
}

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkId: {}", self.0.as_slice().to_base58())
    }
}

impl AsRef<[u8]> for ChunkId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl CheckSum for ChunkId {}

// impl From<(&Hash256, u32)> for ChunkId {
//     fn from(cx: (&Hash256, u32)) -> Self {
//         let (hash, len) = cx;

//         let mut buff = GenericArray::<u8, U32>::default();

//         // 1byte chunk flag
//         buff[0] = 0b_10000000;

//         // 4bytes chunk len
//         buff[1..5].copy_from_slice(&len.to_be_bytes());

//         // remain hash value
//         buff[5..].copy_from_slice(&hash.as_slice()[5..]);

//         Self(buff)
//     }
// }

impl From<(Hash256, u32)> for ChunkId {
    fn from(cx: (Hash256, u32)) -> Self {
        let (hash, len) = cx;

        let mut buff: GenericArray<u8, U32> = hash.into();

        // 1byte chunk flag
        buff[0] = 0b_10000000;

        // 4bytes chunk len
        unsafe {
            let len_ptr = buff[1..5].as_mut_ptr() as * mut u32;
            *len_ptr = len;
        }
       
        // remaining, reserve

        Self(buff)
    }
}

impl ChunkId {
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn len(&self) -> usize {
        let chunkid = self.as_slice();

        return unsafe { * ( chunkid[1..5].as_ptr() as *const u32 ) } as usize;
    }

    pub fn to_objectid(&self) -> ObjectId {
        ObjectId::from(&self.0)
    }

    pub fn to_string58(&self) -> String {
        self.0.as_slice().to_base58()
    }
}

impl RawFixedBytes for ChunkId {
    fn raw_bytes() -> usize {
        GenericArray::<u8, U32>::raw_bytes()
    }
}

impl Serialize for ChunkId {
    fn raw_capacity(&self) -> usize {
        self.0.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> crate::NearResult<&'a mut [u8]> {
        self.0.serialize(buf)
    }
}

impl Deserialize for ChunkId {
    fn deserialize<'de>(buf: &'de [u8]) -> crate::NearResult<(Self, &'de [u8])> {
        let (chunk, buf) = GenericArray::<u8, U32>::deserialize(buf)?;

        Ok((Self(chunk), buf))
    }
}

mod test {

    #[test]
    fn test() {
        use crate::hash_data;
        use super::ChunkId;
        use crate::CheckSum;
        use crate::Serialize;

        let src = "a2245687935115448973";
        let h = hash_data(src.as_bytes());

        println!("{}", ChunkId::default().raw_capacity());

        let id = ChunkId::from((h, src.len() as u32));

        println!("{}, {}", id, id.len());

        println!("{}", id.check_sum())




    }
}
