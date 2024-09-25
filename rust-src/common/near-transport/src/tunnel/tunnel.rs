
use std::any::Any;

use async_trait;

use near_base::{device::DeviceId, *};
use crate::{package::{PackageBodyTrait, PackageDataSet, PackageHeader, PackageHeaderExt }, InterfaceMetaTrait, };

use super::tcp::Tunnel as TcpTunnel;
use super::udp::Tunnel as UdpTunnel;
 
pub trait TunnelStateTrait<Body: PackageBodyTrait, Output=()> {
    fn on_tunnel_event(&self, head: PackageHeader, head_ext: PackageHeaderExt, body: Body) -> State<Output>;
}

#[derive(PartialEq, Eq)]
pub enum State<Output=()> {
    Connecting, 
    Established(Output), 
    Dead, 
}

#[async_trait::async_trait]
pub trait Tunnel: Send + Sync + std::fmt::Display {
    fn clone_as_tunnel(&self) -> DynamicTunnel;
    fn as_any(&self) -> &dyn Any;
    fn local(&self) -> &Endpoint;
    fn remote(&self) -> &Endpoint;
    fn peer_id(&self) -> &DeviceId;
    fn ptr_eq(&self, other: &DynamicTunnel) -> bool;
    fn reset(&self);
    fn update_time(&self) -> Timestamp;
    fn is_closed(&self) -> bool;

    // async fn send_package(&self, package: PackageDataSet) -> NearResult<()>;
}

// impl 
pub struct DynamicTunnel(Box<dyn Tunnel>);

impl DynamicTunnel {
    pub fn new<T: 'static + Tunnel>(tunnel: T) -> Self {
        Self(Box::new(tunnel))
    }

    pub fn clone_as_tunnel<T: 'static + Tunnel + Clone>(&self) -> T {
        self.0.as_any().downcast_ref::<T>().unwrap().clone()
    }

    pub async fn post_message(&self, package: PackageDataSet) -> NearResult<()> {
        if self.local().is_tcp() {
            self.clone_as_tunnel::<TcpTunnel>().send_package(package).await
        } else if self.local().is_udp() {
            self.clone_as_tunnel::<UdpTunnel>().send_package(package).await
        } else {
            unreachable!("don't reach here.")
        }
    }
}

impl AsRef<TcpTunnel> for DynamicTunnel {
    fn as_ref(&self) -> &TcpTunnel {
        self.as_any().downcast_ref::<TcpTunnel>().unwrap()
    }
}

impl AsRef<UdpTunnel> for DynamicTunnel {
    fn as_ref(&self) -> &UdpTunnel {
        self.as_any().downcast_ref::<UdpTunnel>().unwrap()
    }
}

impl std::ops::Deref for DynamicTunnel {
    type Target = dyn Tunnel;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Clone for DynamicTunnel {
    fn clone(&self) -> Self {
        if self.local().is_tcp() {
            Self::new(self.clone_as_tunnel::<TcpTunnel>())
        } else if self.local().is_udp() {
            Self::new(self.clone_as_tunnel::<UdpTunnel>())
        }else {
            unreachable!()
        }
    }
}

impl std::fmt::Display for DynamicTunnel {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.local().is_tcp() {
            (self.as_ref() as &TcpTunnel).fmt(f)
        } else if self.local().is_udp() {
            (self.as_ref() as &UdpTunnel).fmt(f)
        }else {
            unreachable!()
        }
    }

}

impl InterfaceMetaTrait for DynamicTunnel {
    fn clone_as_interface(&self) -> Box<dyn InterfaceMetaTrait> {
        Box::new(self.clone())
    }

    fn local_endpoint(&self) -> Endpoint {
        self.local().clone()
    }

    fn remote_endpoint(&self) -> Endpoint {
        self.remote().clone()
    }
}
