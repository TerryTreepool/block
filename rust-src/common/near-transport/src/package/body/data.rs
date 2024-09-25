
use near_base::{Serialize, Deserialize};

use crate::package::PackageBodyTrait;

#[derive(Default)]
pub struct Data {
    data: Vec<u8>,
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "data-size={}", self.data.len())
    }
}

impl Data {
    pub fn with_data(data: Vec<u8>) -> Self {
        Self {
            data: data,
        }
    }
}

impl From<Data> for Vec<u8> {
    fn from(data: Data) -> Self {
        data.data
    }
}

impl From<Vec<u8>> for Data {
    fn from(data: Vec<u8>) -> Self {
        Self {
            data: data,
        }
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl Serialize for Data {
    fn raw_capacity(&self) -> usize {
        self.data.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        self.data.serialize(buf)
    }
}

impl Deserialize for Data {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        let (data, buf) = Vec::<u8>::deserialize(buf)?;

        Ok((Self {data}, buf))
    }
}

impl PackageBodyTrait for Data {
    fn version() -> u8 {
        1u8
    }
}
