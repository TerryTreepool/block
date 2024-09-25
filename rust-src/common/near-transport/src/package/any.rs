
use crate::package::{MajorCommand, PackageBodyTrait};

use near_base::*;

use super::{Ack, AckAck, AckAckTunnel, AckTunnel, Data, Exchange, StunReq};

#[derive(Clone)]
pub struct Request<T>(T);

impl<T: PackageBodyTrait> Request<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> From<T> for Request<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: PackageBodyTrait> Serialize for Request<T> {
    fn raw_capacity(&self) -> usize {
        self.0.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.0.serialize(buf)
    }

}

impl<T: PackageBodyTrait + Deserialize> Deserialize for Request<T> {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (data, buf) = T::deserialize(buf)?;

        Ok((Self::new(data), buf))
    }
}

impl<T: PackageBodyTrait> PackageBodyTrait for Request<T> {
    fn version() -> u8 {
        T::version()
    }
}

#[derive(Clone)]
pub struct Response<T>(T);

impl<T: PackageBodyTrait> Response<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

}

impl<T: PackageBodyTrait> Serialize for Response<T> {
    fn raw_capacity(&self) -> usize {
        self.0.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.0.serialize(buf)
    }

}

impl<T: PackageBodyTrait + Deserialize> Deserialize for Response<T> {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (data, buf) = T::deserialize(buf)?;

        Ok((Self::new(data), buf))
    }
}

impl<T: PackageBodyTrait> PackageBodyTrait for Response<T> {
    fn version() -> u8 {
        T::version()
    }
}

impl<T> From<T> for Response<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}


// #[derive(Clone)]
// pub struct CallRequest<T>(T);

// impl<T: PackageBodyTrait> CallRequest<T> {
//     pub fn new(value: T) -> Self {
//         Self(value)
//     }

// }

// impl<T: PackageBodyTrait> Serialize for CallRequest<T> {
//     fn raw_capacity(&self) -> usize {
//         self.0.raw_capacity()
//     }

//     fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
//         self.0.serialize(buf)
//     }

// }

// impl<T: PackageBodyTrait + Deserialize> Deserialize for CallRequest<T> {
//     fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
//         let (data, buf) = T::deserialize(buf)?;

//         Ok((Self::new(data), buf))
//     }
// }

// impl<T: PackageBodyTrait> PackageBodyTrait for CallRequest<T> {
//     fn version() -> u8 {
//         T::version()
//     }
// }
// type CallRequest<T> = Request<T>;

#[derive(Default)]
pub enum AnyNamedRequest {
    #[default]
    None,
    Exchange(Exchange),
    AckTunnel(AckTunnel),
    AckAckTunnel(AckAckTunnel),
    Ack(Ack),
    AckAck(AckAck),
    Stun(StunReq),
    Request(Request<Data>),
    Response(Response<Data>),
}

impl AnyNamedRequest {

    pub fn with_exchange(data: Exchange) -> Self {
        Self::Exchange(data)
    }

    pub fn with_acktunnel(data: AckTunnel) -> Self {
        Self::AckTunnel(data)
    }

    pub fn with_ackacktunnel(data: AckAckTunnel) -> Self {
        Self::AckAckTunnel(data)
    }

    pub fn with_ack(data: Ack) -> Self {
        Self::Ack(data)
    }

    pub fn with_ackack(data: AckAck) -> Self {
        Self::AckAck(data)
    }

    pub fn with_stun(data: StunReq) -> Self {
        Self::Stun(data)
    }

    pub fn with_request(data: Data) -> Self {
        Self::Request(Request::new(data))
    }

    pub fn with_response(data: Data) -> Self {
        Self::Response(Response::new(data))
    }

    pub fn major_command(&self) -> MajorCommand {
        match self {
            Self::None => { MajorCommand::None },
            Self::Exchange(_) => { MajorCommand::Exchange },
            Self::AckTunnel(_) => { MajorCommand::AckTunnel },
            Self::AckAckTunnel(_) => { MajorCommand::AckAckTunnel },
            Self::Ack(_) => { MajorCommand::Ack },
            Self::AckAck(_) => { MajorCommand::AckAck },
            Self::Stun(_) => { MajorCommand::Stun },
            Self::Request(_) => { MajorCommand::Request },
            Self::Response(_) => { MajorCommand::Response },
        }
    }
}

impl Serialize for AnyNamedRequest {
    fn raw_capacity(&self) -> usize {
        match self {
            Self::None => { 0 },
            Self::Exchange(v) => { v.raw_capacity() },
            Self::AckTunnel(v) => { v.raw_capacity() },
            Self::AckAckTunnel(v) => { v.raw_capacity() },
            Self::Ack(v) => { v.raw_capacity() },
            Self::AckAck(v) => { v.raw_capacity() }
            Self::Stun(v) => { v.raw_capacity() },
            Self::Request(v) => { v.raw_capacity() },
            Self::Response(v) => { v.raw_capacity() },
        }

    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        match self {
            Self::None => { Ok(buf) },
            Self::Exchange(v) => { v.serialize(buf) },
            Self::AckTunnel(v) => { v.serialize(buf) },
            Self::AckAckTunnel(v) => { v.serialize(buf) },
            Self::Ack(v) => { v.serialize(buf) },
            Self::AckAck(v) => { v.serialize(buf) }
            Self::Stun(v) => { v.serialize(buf) },
            Self::Request(v) => { v.serialize(buf) },
            Self::Response(v) => { v.serialize(buf) },
        }
    }
}

impl std::fmt::Display for AnyNamedRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Exchange(_) => write!(f, "Exchange"),
            Self::AckTunnel(_) => write!(f, "AckTunnel"),
            Self::AckAckTunnel(_) => write!(f, "AckAckTunnel"),
            Self::Ack(_) => write!(f, "Ack"),
            Self::AckAck(_) => write!(f, "AckAck"),
            Self::Stun(stun) => write!(f, "Stun-{}", stun.stun_name()),
            Self::Request(_) => write!(f, "Request"),
            Self::Response(_) => write!(f, "Response"),
        }
    }
}
