
use std::sync::atomic::{AtomicU32, Ordering};
use generic_array::{GenericArray, typenum::U32};
use rand::random;

use crate::{Serialize, Deserialize, RawFixedBytes};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SequenceValue(u32);

impl SequenceValue {
    pub fn into_value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for SequenceValue {
    fn from(v: u32) -> Self {
        SequenceValue(v)
    }
}

impl std::default::Default for SequenceValue {
    fn default() -> Self {
        Self(0)
    }
}

pub struct Sequence(AtomicU32);

impl Sequence {
    pub fn random() -> Self {
        Self(AtomicU32::new(random::<u32>()))
    }

    pub fn generate(&self) -> SequenceValue {
        SequenceValue::from(self.0.fetch_add(1, Ordering::SeqCst) + 1)
    }

    pub fn into_value(&self) -> SequenceValue {
        SequenceValue::from(self.0.load(Ordering::SeqCst))
    }
}

impl Clone for Sequence {
    fn clone(&self) -> Self {
        Sequence::from(&self.into_value())
    }
}

impl From<u32> for Sequence {
    fn from(v: u32) -> Self {
        Self(AtomicU32::new(v))
    }
}

impl From<&SequenceValue> for Sequence {
    fn from(v: &SequenceValue) -> Self {
        Self(AtomicU32::new(v.into_value()))
    }
}

impl std::default::Default for Sequence {
    fn default() -> Self {
        Self(AtomicU32::new(0))
    }
}

/// sequence string
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SequenceString(GenericArray<u8, U32>);

impl std::fmt::Display for SequenceString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self))
    }
}

impl std::fmt::Debug for SequenceString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Display).fmt(f)
    }
}

impl AsRef<[u8]> for SequenceString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<&[u8; 32]> for SequenceString {
    fn from(value: &[u8; 32]) -> Self {
        Self(GenericArray::clone_from_slice(value))
    }
}

impl From<&[u8]> for SequenceString {
    fn from(value: &[u8]) -> Self {
        let mut ret = Self::default();
        ret.0.as_mut_slice().copy_from_slice(&value);
        ret
    }
}

impl RawFixedBytes for SequenceString {
    fn raw_bytes() -> usize {
        GenericArray::<u8, U32>::raw_bytes()
    }
}

impl Serialize for SequenceString {
    fn raw_capacity(&self) -> usize {
        self.0.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> crate::NearResult<&'a mut [u8]> {
        self.0.serialize(buf)
    }
}

impl Deserialize for SequenceString {
    fn deserialize<'de>(buf: &'de [u8]) -> crate::NearResult<(Self, &'de [u8])> {
        let (r, buf) = GenericArray::<u8, U32>::deserialize(buf)?;

        Ok((Self(r), buf))
    }
}
