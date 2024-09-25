
use near_base::*;

use super::Interface;

// pub trait NetInterfaceCallback: Send + Sync {
//     fn clone_as_callback(&self) -> Box<dyn NetInterfaceCallback>;
//     fn on_package(&self, data: DataContext) -> NearResult<()>;
// }

pub trait NetInterface: Send + Sync {
    fn clone_as_interface(&self) -> Box<dyn NetInterface>;
    fn clone_into_interface(&self) -> Box<dyn Interface>;
    fn start(&self) -> NearResult<()>;
}

// use async_std::net::{TcpStream as AsyncTcpStream, UdpSocket as AsyncUdpSocket};

// use super::test::DynamicPackage;

// enum Socket {
//     Tcp(AsyncTcpStream),
//     Udp(AsyncUdpSocket),
// }

// struct NetDataSourceImpl {
//     socket: Socket,
//     endpoint_pair: EndpointPair,
//     package: DynamicPackage,
// }

// #[derive(Clone)]
// pub struct DataContext(Arc<NetDataSourceImpl>);

// impl DataContext {
//     pub fn with_tcpstream(stream: AsyncTcpStream, endpoint: EndpointPair, package: DynamicPackage) -> Self {
//         Self(Arc::new(NetDataSourceImpl{
//             socket: Socket::Tcp(stream),
//             endpoint_pair: endpoint,
//             package: package,
//         }))
//     }

//     pub fn with_udpsocket(socket: AsyncUdpSocket, endpoint: EndpointPair, package: DynamicPackage) -> Self {
//         Self(Arc::new(NetDataSourceImpl{
//             socket: Socket::Udp(socket), 
//             endpoint_pair: endpoint,
//             package: package,
//         }))
//     }

//     pub fn local(&self) -> &Endpoint {
//         self.0.endpoint_pair.local()
//     }

//     pub fn remote(&self) -> &Endpoint {
//         self.0.endpoint_pair.remote()
//     }

//     pub fn package(&self) -> &DynamicPackage {
//         &self.0.package
//     }
// }
