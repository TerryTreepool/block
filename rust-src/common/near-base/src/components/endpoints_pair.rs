
use crate::{Deserialize, Serialize};

use super::Endpoint;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EndpointPair {
    local: Endpoint,
    remote: Endpoint,
}

impl EndpointPair {
    pub fn new(local: Endpoint, remote: Endpoint) -> Self {
        Self { local, remote }
    }

    #[inline]
    pub fn local(&self) -> &Endpoint {
        &self.local
    }

    #[inline]
    pub fn remote(&self) -> &Endpoint {
        &self.remote
    }

    #[inline]
    pub fn is_tcp(&self) -> bool {
        self.local.is_tcp()
    }

    #[inline]
    pub fn is_udp(&self) -> bool {
        self.local.is_udp()
    }

    #[inline]
    pub fn split(self) -> (Endpoint, Endpoint) {
        (self.local, self.remote)
    }
}

impl std::fmt::Display for EndpointPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[local: [{}], remote: [{}]]", self.local, self.remote)
    }
}

impl std::fmt::Debug for EndpointPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Display).fmt(f)
    }
}

impl Serialize for EndpointPair {
    fn raw_capacity(&self) -> usize {
        self.local.raw_capacity() + 
        self.remote.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> crate::NearResult<&'a mut [u8]> {
        let buf = self.local.serialize(buf)?;
        let buf = self.remote.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for EndpointPair {
    fn deserialize<'de>(buf: &'de [u8]) -> crate::NearResult<(Self, &'de [u8])> {
        let (local, buf) = Endpoint::deserialize(buf)?;
        let (remote, buf) = Endpoint::deserialize(buf)?;

        Ok((Self {
            local, remote,
        }, buf))
    }
}
