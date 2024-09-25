
use std::{sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}}, 
          time::Duration, 
    };

use async_std::{net::TcpStream as AsyncTcpStream, io::WriteExt};

use log::{debug, info, error, trace};

use near_base::*;

use super::{Interface as InterfaceTrait, PackageDecodeTrait, State};

struct InterfaceImpl {
    socket: AsyncTcpStream,
    local: Endpoint,
    remote: Endpoint,
    is_closed: AtomicBool,
    state: RwLock<State>,
}

impl std::ops::Drop for InterfaceImpl {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            // use std::os::windows::io::AsRawSocket;
            // use winapi::um::winsock2::closesocket;
            // unsafe {
            //     let raw = self.socket.as_raw_socket();
            //     closesocket(raw.try_into().unwrap());
            // }
        }
        #[cfg(not(windows))]
        {
            let _ = self.socket.shutdown(std::net::Shutdown::Both);
            use std::os::fd::AsRawFd;
            unsafe {
                let raw = self.socket.as_raw_fd();
                libc::close(raw);
            }
        }
    }
}

impl std::fmt::Display for InterfaceImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(target_os="windows")]
        {
            use std::os::windows::prelude::AsRawSocket;
            write!(f, "TcpInterface{{local:{}, remote:{}, socket-fd:{}}}", self.local, self.remote, self.socket.as_raw_socket())
        }

        #[cfg(not(target_os="windows"))]
        {
            use std::os::fd::AsRawFd;
            write!(f, "TcpInterface{{local:{}, remote:{}, socket-fd:{}}}", self.local, self.remote, self.socket.as_raw_fd())
        }
    }
}

#[derive(Clone)]
pub struct Interface(Arc<InterfaceImpl>);

impl std::fmt::Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[async_trait::async_trait]
impl InterfaceTrait for Interface {
    fn clone_as_interface(&self) -> Box<dyn InterfaceTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn local(&self) ->  &Endpoint {
        &self.0.local
    }

    fn state(&self) -> State {
        self.0.state.read().unwrap().clone()
    }

    fn is_closed(&self) -> bool {
        self.0.is_closed.load(Ordering::SeqCst)
    }

    // async fn send_data(&self, send_buf: &[u8], _: Option<Endpoint>) ->  NearResult<()> {
    //     self.send_data(send_buf).await
    // }

}

impl Interface {
    // pub async fn send_package(&self,
    //                           package: &mut DynamicPackage,
    //                           signer: Option<impl SignerTrait>) -> NearResult<()> {
    //     let mut send_buf = [0u8; MTU];
    //     let end_buf = package.serialize(&mut send_buf, signer).await?;
    //     let len = MTU - end_buf.len();

    //     self.send_data(&send_buf[0..len]).await
    // }

    pub async fn send_data(
        &self,
        send_buf: &[u8]
    ) -> NearResult<()> {
        let mut socket = self.0.socket.clone();

        let _ = socket.write_all(send_buf)
                    .await
                    .map_err(|err| {
                        let error_string = format!("faile write_all to {} with e = {err}", self);
                        error!("{error_string}");
                        NearError::from(err)
                    })?;

        Ok(())
    }

    pub fn socket(&self) -> &AsyncTcpStream {
        &self.0.socket
    }

    pub fn local(&self) -> &Endpoint {
        &self.0.local
    }

    pub fn remote(&self) -> &Endpoint {
        &self.0.remote
    }

    pub fn endpoint_pair(&self) -> EndpointPair {
        EndpointPair::new(self.local().clone(), self.remote().clone())
    }
}

impl Interface {
    pub(in super::super) async fn connect(
        remote: &Endpoint,
        timeout: Duration
    ) -> NearResult<Self> {
        let remote_addr = {
            if remote.is_tcp() {
                remote.addr()
            } else {
                unreachable!()
            }
        };

        let socket =
            async_std::future::timeout(timeout, AsyncTcpStream::connect(remote_addr))
                .await
                .map_err(|err| {
                    let error_string = format!("failed connect with err {}", err);
                    error!("{}", error_string);
                    NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string)
                })??;

        let local = socket.local_addr().map_err(|err| { NearError::from(err) } )?;
        let local = Endpoint::default_tcp(local);

        let interface = Interface(Arc::new(InterfaceImpl{
            socket,
            local,
            remote: remote.clone(),
            is_closed: AtomicBool::new(false),
            state: RwLock::new(State::Active(now())),
        }));
        debug!("{} connected", interface);

        Ok(interface)
    }

    pub(in super::super) async fn accept(socket: AsyncTcpStream) -> NearResult<Self> {
        let local = socket.local_addr().map_err(|err| NearError::from(err) )?;
        let remote = socket.peer_addr().map_err(|err| NearError::from(err) )?;

        let interface = Interface(Arc::new(InterfaceImpl{
            socket,
            local: Endpoint::default_tcp(local),
            remote: Endpoint::default_tcp(remote),
            is_closed: AtomicBool::new(false),
            state: RwLock::new(State::Active(now())),
        }));

        debug!("{} accepted", interface);

        Ok(interface)
    }

    pub(in super::super) fn close(&self) {
        trace!("{} will close", self);

        let to_close = {
            let state = &mut *self.0.state.write().unwrap();
            match state {
                State::Active(timestamp) => {
                    let shutdown_stamp = now();
                    info!("[{}] was dead at {}.", self, shutdown_stamp);
                    *state = State::Closed(*timestamp, shutdown_stamp);
                    true
                }
                _ => false,
            }
        };

        if to_close {
            let _ = self.0.is_closed.store(true, Ordering::SeqCst);

            // socket fd don't close in there.
            // when interfaceimpl dropping it will close.
        }
    }
}

impl Interface {
    pub(in super::super) async fn recv_package<R>(&self, decoder: impl PackageDecodeTrait<R=R>) -> NearResult<R> {
        decoder.package_decode(self.socket()).await
    }
}
