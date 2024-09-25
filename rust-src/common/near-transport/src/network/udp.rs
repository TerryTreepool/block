
use std::sync::Arc;

use log::error;
use near_base::*;

use super::{data_context::NetInterface, interface::{PackageDecodeDef, State}, Interface, NetManager, UdpInterface, UdpPackageEventTrait};

struct UdpImpl {
    manager: NetManager,
    udp: UdpInterface,
}

#[derive(Clone)]
pub struct Udp(Arc<UdpImpl>);

impl Udp {
    pub fn bind(manager: NetManager, local: Endpoint) -> NearResult<Self> {
        if !local.is_udp() {
            unreachable!("must bind UDP protocol")
        }

        let listener = UdpInterface::bind(Some(local))?;

        Ok(Self(Arc::new(UdpImpl {
            manager,
            udp: listener,
        })))
    }

    pub async fn connect(&self, remote_endpoint: &Endpoint, remote: DeviceObject) -> NearResult<()> {
        if !remote_endpoint.is_udp() {
            unreachable!("must connect UDP protocol")
        }

        match self.0.udp.state() {
            State::Active(_) => {
                self.0.manager.stack().on_connected(self.0.udp.clone(), &remote, remote_endpoint.clone());
                Ok(())
            }
            _ => {
                error!("tunnel is closed.");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TUNNEL_CLOSED, "tunnel is closed."))
            }
        }
    }
    // pub async fn connect(manager: NetManager, remote_endpoint: &Endpoint, remote: DeviceObject) -> NearResult<Self> {
    //     if !remote_endpoint.is_udp() {
    //         unreachable!("must connect UDP protocol")
    //     }

    //     let udp = UdpInterface::bind(None)?;

    //     Ok(Self(Arc::new(UdpImpl {
    //         manager,
    //         udp: UdpState::Connector(ConnectState{
    //             remote_ep: remote_endpoint.clone(),
    //             remote,
    //             connector: udp,
    //         }),
    //     })))
    // }

    #[allow(unused)]
    pub fn local_address(&self) -> &Endpoint {
        self.0.udp.local_address()
    }

}

impl NetInterface for Udp {
    fn clone_as_interface(&self) -> Box<dyn NetInterface> {
        Box::new(self.clone())
    }

    fn clone_into_interface(&self) -> Box<dyn Interface> {
        self.0.udp.clone_as_interface()
    }


    fn start(&self) -> NearResult<()> {
        let this = self.clone();

        async_std::task::spawn(async move {
            let listener = &this.0.udp;

            loop {
                match listener.recv_package(PackageDecodeDef).await {
                    Ok((remote, package)) => {
                        let this = this.clone();
                        let listener: UdpInterface = listener.clone();
                        async_std::task::spawn(async move {
                            let _ = 
                                this.0
                                    .manager.stack()
                                    .on_udp_package(listener, package, remote)
                                    .await;
                        });
                    }
                    Err(err) => {
                        error!("failed recv_package() with errno: {err}");
                        break;
                    }
                }
            };

            listener.close();
        });

        Ok(())
    }
}
