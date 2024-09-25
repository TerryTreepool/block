
use crate::{components::ObjectTypeCode, 
            Serialize, Deserialize,
            public_key::PublicKey,
            errors::*, ObjectBodyTrait, ObjectDescTrait,
            Area,
            Hash256, ChunkId, };

use super::{object_type::ObjectId,
            object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody},};

pub type FileDesc = NamedObjectDesc<FileDescContent>;
pub type FileBody = NamedObjectBody<FileBodyContent>;
pub type FileObject = NamedObject<FileDescContent, FileBodyContent>;

#[derive(Clone, Default)]
pub struct FileDescContent {
    name: String,
    len: u64,
    hash: Hash256,
}

impl FileDescContent {
    pub fn new(name: String, len: u64, hash: Hash256) -> Self {
        Self { name, len, hash }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn hash(&self) -> &Hash256 {
        &self.hash
    }
}

impl std::fmt::Display for FileDescContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "object_type_code: [{}], name: [{}], len: [{}], hash: [{}]", self.object_type_code(), self.name, self.len, self.hash)
    }
}

impl ObjectDescTrait for FileDescContent {
    fn object_type_code(&self) -> ObjectTypeCode {
        ObjectTypeCode::with_file()
    }

    type OwnerObj = ObjectId;
    type AreaObj = Area;
    type AuthorObj = ObjectId;
    type PublicKeyObj = PublicKey;

}

impl Serialize for FileDescContent {
    fn raw_capacity(&self) -> usize {
        self.name.raw_capacity() +
        self.len.raw_capacity() + 
        self.hash.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.name.serialize(buf)?;
        let buf = self.len.serialize(buf)?;
        let buf = self.hash.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for FileDescContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (name, buf) = String::deserialize(buf)?;
        let (len, buf) = u64::deserialize(buf)?;
        let (hash, buf) = Hash256::deserialize(buf)?;

        Ok((Self{
            name, len, hash, 
        }, buf))
    }

}

#[derive(Clone, Default)]
pub struct FileBodyContent {
    chunk_list: Vec<ChunkId>,
}

impl FileBodyContent {
    pub fn new(chunk_list: Vec<ChunkId>) -> Self {
        Self {
            chunk_list
        }
    }
}

impl FileBodyContent {
    pub fn chunk_list(&self) -> &[ChunkId] {
        self.chunk_list.as_slice()
    }

    pub fn mut_chunk_list(&mut self) -> &mut Vec<ChunkId> {
        &mut self.chunk_list
    }
}

impl std::fmt::Display for FileBodyContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileBodyContent: [ chunk_list: {:?} ]", 
               self.chunk_list)
    }
}

impl ObjectBodyTrait for FileBodyContent { 
}

impl Serialize for FileBodyContent {
    fn raw_capacity(&self) -> usize {
        self.chunk_list.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.chunk_list.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for FileBodyContent {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (chunk_list, buf) = Vec::<ChunkId>::deserialize(buf)?;

        Ok((Self{
            chunk_list
        }, buf))
    }

}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use crate::{hash_data, Area, FileEncoder };

    use super::*;
    use crate::ObjectBuilder;

    #[test]
    fn test_file() {
        let file = 
            ObjectBuilder::new(FileDescContent::new("test".into(), "123".len() as u64, hash_data("123".as_bytes())), 
                               FileBodyContent::new(vec![]))
                .update_desc(|desc| {
                    desc.set_area(Some(Area::new(123, 10, 2321, 10)));
                })
                .update_body(|_body| {
                })
                .build().unwrap();

        println!("core-servic: {}", file);

        println!("{:?}", file.encode_to_file(PathBuf::new().join("test_file.desc").as_path(), false));
    }

}


