
use near_base::{file::FileObject, Deserialize, Serialize, NearResult, NearError};
use near_transport::ItfTrait;

// sync protocol
#[derive(Clone, Default)]
pub struct SyncFileMessage {
    pub file: FileObject,
}

impl Serialize for SyncFileMessage {
    fn raw_capacity(&self) -> usize {
        self.file.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.file.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for SyncFileMessage {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (file, buf) = FileObject::deserialize(buf)?;

        Ok((Self{
            file,
        }, buf))
    }

}

impl From<FileObject> for SyncFileMessage {
    fn from(v: FileObject) -> Self {
        Self {
            file: v
        }
    }
}

impl std::fmt::Display for SyncFileMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.file.fmt(f)
    }
}

// pub struct SyncFileMessageResponse {
//     pub errno: NearError,
// }

// impl ItfTrait for SyncFileMessageResponse {}

// impl Serialize for SyncFileMessageResponse {
//     fn raw_capacity(&self) -> usize {
//         self.errno.raw_capacity()
//     }

//     fn serialize<'a>(&self,
//                      buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
//         self.errno.serialize(buf)
//     }

// }

// impl Deserialize for SyncFileMessageResponse {
//     fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
//         let (errno, buf) = NearError::deserialize(buf)?;

//         Ok((Self{
//             errno
//         }, buf))
//     }

// }
