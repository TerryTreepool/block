
mod tcp;
mod udp;

use std::{ops::Deref, any::Any};

use async_std::io::Read;

pub use tcp::Interface as TcpInterface;
pub use udp::Interface as UdpInterface;

use near_base::*;

use crate::package::package_decode;

use super::DataContext;

#[derive(Clone)]
pub enum State {
    Active(Timestamp),
    Closed(Timestamp /* active timestamp */, Timestamp /* dead timestamp */),
}

#[async_trait::async_trait]
pub trait Interface: Send + Sync {
    fn clone_as_interface(&self) -> Box<dyn Interface>;
    fn as_any(&self) -> &dyn Any;
    fn local(&self) -> &Endpoint;
    fn state(&self) -> State;
    fn is_closed(&self) -> bool;
    // async fn send_data(&self, send_buf: &[u8], remote: Option<Endpoint>) ->  NearResult<()>;
}

pub struct DynamicInterface(Box<dyn Interface>);

impl DynamicInterface {
    pub fn new<I: 'static + Interface>(interface: I) -> Self {
        Self(Box::new(interface))
    }
}

impl AsRef<TcpInterface> for DynamicInterface {
    fn as_ref(&self) -> &TcpInterface {
        self.as_any().downcast_ref::<TcpInterface>().unwrap()
    }
}

impl AsRef<UdpInterface> for DynamicInterface {
    fn as_ref(&self) -> &UdpInterface {
        self.as_any().downcast_ref::<UdpInterface>().unwrap()
    }
}

impl Deref for DynamicInterface {
    type Target = dyn Interface;

    /// Dereferences the value.
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl std::fmt::Display for DynamicInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.deref().local().is_tcp() {
            self.as_any().downcast_ref::<TcpInterface>().unwrap().fmt(f)
        } else if self.deref().local().is_udp() {
            self.as_any().downcast_ref::<UdpInterface>().unwrap().fmt(f)
        } else {
            unreachable!("don't reach here.")
        }
    }
}

impl std::clone::Clone for DynamicInterface {
    fn clone(&self) -> Self {
        if self.deref().local().is_tcp() {
            DynamicInterface::new(self.as_any().downcast_ref::<TcpInterface>().unwrap().clone())
        } else if self.deref().local().is_udp() {
            DynamicInterface::new(self.as_any().downcast_ref::<UdpInterface>().unwrap().clone())
        } else {
            unreachable!("don't reach here.")
        }
    }
}

#[async_trait::async_trait]
pub trait PackageDecodeTrait {
    type R;
    async fn package_decode<IO: Read + Unpin + Send>(&self, io: IO) -> NearResult<Self::R>;
}

pub struct PackageDecodeDef;

#[async_trait::async_trait]
impl PackageDecodeTrait for PackageDecodeDef {
    type R = DataContext;
    async fn package_decode<IO: Read + Unpin + Send>(&self, io: IO) -> NearResult<DataContext> {
        package_decode::decode_package(io).await
    }
}

// #[async_trait::async_trait]
// impl<F, IO> PackageDecodeTrait for F
// where
//     F: Send + Sync + 'static + Fn(IO) -> NearResult<DataContext>,
//     IO: Read + Unpin,

// {
//     async fn package_decode(&self, io: IO) -> NearResult<DataContext> {
//         let fut = (self)(io);
//         let res = fut?;
//         Ok(res)
//     }
// }