
use std::{sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}}, net::{SocketAddr, IpAddr, Ipv4Addr}};

use async_std::net::UdpSocket as AsyncUdpSocket;

use log::info;

use near_base::*;

use super::{Interface as InterfaceTrait, PackageDecodeTrait, State};

struct InterfaceImpl {
    socket: AsyncUdpSocket,
    local: Endpoint,
    is_closed: AtomicBool,
    state: RwLock<State>
}

#[derive(Clone)]
pub struct Interface(Arc<InterfaceImpl>);

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

    // async fn send_data(
    //     &self, 
    //     send_buf: &[u8], 
    //     remote: Option<Endpoint>
    // ) ->  NearResult<()> {
    //     if let Some(remote) = remote.as_ref() {
    //         self.send_data_to(send_buf, remote).await
    //     } else {
    //         Err(NearError::new(ErrorCode::NEAR_ERROR_NO_TARGET, "remote cannot nil."))
    //     }
    // }

}

impl std::fmt::Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UdpInterface{{local:{}, }}", self.0.local,)
    }
}

impl Interface {
    pub(in crate) fn bind(local: Option<Endpoint>) -> NearResult<Interface> {

        let local = local.unwrap_or(Endpoint::default_udp(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)));

        if !local.is_udp() {
            unreachable!("must bind udp protocol")
        }

        let listener = {
            if local.is_ipv6() {
                #[cfg(windows)]
                {
                    std::net::UdpSocket::bind(&local)
                        .map_err(| err | {
                            NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, 
                                format!("failed bind() ipv6 with errno({})", err))
                        })
                }
                #[cfg(not(windows))]
                {
                    unimplemented!()
                }
            } else {
                // ipv4
                std::net::UdpSocket::bind(&local)
                    .map_err(|err| {
                        NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, 
                            format!("failed bind() ipv4 with errno({})", err))
                    })
            }
        }?;

        let local_address = listener.local_addr().map(| addr | Endpoint::default_udp(addr)).unwrap_or(local);

        Ok(Interface(Arc::new(InterfaceImpl{
            socket: AsyncUdpSocket::from(listener),
            local: local_address,
            is_closed: AtomicBool::new(false),
            state: RwLock::new(State::Active(now()))
        })))
    }

    #[allow(unused)]
    pub fn local_address(&self) -> &Endpoint {
        &self.0.local
    }

    pub fn socket(&self) -> &AsyncUdpSocket {
        &self.0.socket
    }

    pub fn close(&self) {
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
            self.0.is_closed.store(true, Ordering::SeqCst);

            #[cfg(windows)]
            {
                use std::os::windows::io::AsRawSocket;
                use winapi::um::winsock2::closesocket;
                unsafe {
                    let raw = self.0.socket.as_raw_socket();
                    closesocket(raw.try_into().unwrap());
                }
            }
            #[cfg(not(windows))]
            {
                use std::os::unix::io::AsRawFd;
                unsafe {
                    let raw = self.0.socket.as_raw_fd();
                    libc::close(raw);
                }
            }
        }
    }

}

impl Interface {

    #[allow(unused)]
    pub async fn send_data_to(&self, send_buf: &[u8], to: &Endpoint) -> NearResult<()> {
        self.0.socket
            .send_to(send_buf, to.addr())
            .await
            .map(|len| () )
            .map_err(|err| NearError::from(err) )
    }

    pub(in crate) async fn recv_package<R>(&self, decoder: impl PackageDecodeTrait<R=R>) -> NearResult<(Endpoint, R)> {

        use crate::network::MTU;

        let mut recv_buf = [0u8; MTU];

        loop {
            match self.socket().recv_from(&mut recv_buf).await {
                Ok((size, remote)) => {
                    let package = decoder.package_decode(&recv_buf[..size]).await?;
                    break Ok((Endpoint::default_udp(remote), package))
                }
                Err(err) => {
                    if let Some(10054i32) = err.raw_os_error() {
                        // In Windows, if host A use UDP socket and call sendto() to send something to host B,
                        // but B doesn't bind any port so that B doesn't receive the message,
                        // and then host A call recvfrom() to receive some message,
                        // recvfrom() will failed, and WSAGetLastError() will return 10054.
                        // It's a bug of Windows.
                        /* trace!("{} socket recv failed for {}, ignore this error", self, err); */
                    } else {
                        // info!("{} socket recv failed for {}, break recv loop", self, err);
                        break Err(NearError::from(err))
                    }
                }
            }
        }
    }

    // pub(in super::super) async fn recv_package(&self) -> NearResult<(Endpoint, DataContext)> {
    //     use crate::network::MTU;

    //     let mut recv_buf = [0u8; MTU];

    //     loop {
    //         match self.socket().recv_from(&mut recv_buf).await {
    //             Ok((size, remote)) => {
    //                 let package = 
    //                     package_decode::decode_package(&recv_buf[..size]).await?;
    //                     // DataInterface::with_datagram(recv_buf, size)
    //                     //     .recv_package()
    //                     //     .await?;
    //                 break Ok((Endpoint::default_udp(remote), package))
    //             }
    //             Err(err) => {
    //                 if let Some(10054i32) = err.raw_os_error() {
    //                     // In Windows, if host A use UDP socket and call sendto() to send something to host B,
    //                     // but B doesn't bind any port so that B doesn't receive the message,
    //                     // and then host A call recvfrom() to receive some message,
    //                     // recvfrom() will failed, and WSAGetLastError() will return 10054.
    //                     // It's a bug of Windows.
    //                     /* trace!("{} socket recv failed for {}, ignore this error", self, err); */
    //                 } else {
    //                     // info!("{} socket recv failed for {}, break recv loop", self, err);
    //                     break Err(NearError::from(err))
    //                 }
    //             }
    //         }
    //     }

    // }

}

