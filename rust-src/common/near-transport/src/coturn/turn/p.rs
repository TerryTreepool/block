
use std::{cell::RefCell, sync::Arc};

use async_std::io::{BufReader, Read, ReadExt};

use log::{error, info, trace};
use near_base::{aes_key::KeyMixHash, Deserialize, Endpoint, NearResult};

use crate::{network::{Interface, PackageDecodeTrait, UdpInterface}, PayloadMaxLen};


struct ProxyInterfaceImpl {
    socket: UdpInterface, 
    local: Endpoint, 
    proxy_datagram: Box<dyn ProxyDatagramTrait>,
}

#[async_trait::async_trait]
pub trait ProxyDatagramTrait: Send + Sync {
    async fn on_proxied_datagram(&self, mix_hash: KeyMixHash, datagram: &[u8], from: Endpoint);
}

#[derive(Clone)]
pub struct ProxyInterface(Arc<ProxyInterfaceImpl>);

impl std::fmt::Display for ProxyInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProxyInterface:{{listen:{:?}}}", self.local())
    }
}

thread_local! {
    static UDP_RECV_BUFFER: RefCell<[u8; PayloadMaxLen]> = RefCell::new([0u8; PayloadMaxLen]);
}

impl ProxyInterface {
    pub fn open(local: Option<Endpoint>, proxy_datagram: Box<dyn ProxyDatagramTrait>) -> NearResult<Self> {

        let socket = UdpInterface::bind(local)?;
        let local = socket.local().clone();

        let interface = 
            Self(Arc::new(ProxyInterfaceImpl {
                socket, 
                local,
                proxy_datagram,
            }));

        let num_cpus = num_cpus::get_physical();
        let num_cpus = if num_cpus == 0 { 1 } else { num_cpus };
        let pool_size = num_cpus * 2;
        for _ in 0..pool_size {
            let interface = interface.clone();
            async_std::task::spawn(async move {
                interface.proxy_loop().await;
            });
        }

        Ok(interface)
    }

    pub fn udp(&self) -> UdpInterface {
        self.0.socket.clone()
    }

    pub fn local(&self) -> &Endpoint {
        &self.0.local
    }

    async fn proxy_loop(&self) {
        info!("{} started", self);

        struct ProxyPacketDecode;

        #[async_trait::async_trait]
        impl PackageDecodeTrait for ProxyPacketDecode {
            type R = Vec<u8>;
            async fn package_decode<IO: Read + Unpin + Send>(&self, io: IO) -> NearResult<Vec<u8>> {
                let mut data = vec![0u8; PayloadMaxLen];
                let mut reader = BufReader::new(io);

                let _ = reader.read(data.as_mut_slice()).await?;

                Ok(data)
            }
        }

        loop {
            match   self.0
                        .socket
                        .recv_package(ProxyPacketDecode)
                        .await {
                Ok((remote, data)) => {
                    trace!("{} recv datagram len {} from {}", self, data.len(), remote);
                    self.on_proxied_datagram(data.as_slice(), remote).await;
                }
                Err(err) => {
                    error!("{} socket recv failed for {}, break recv loop", self, err);
                }
            }
        }

        // let this = self.clone();

        // loop {
        //     UDP_RECV_BUFFER.with(|thread_recv_buf| {
        //         let recv_buf = &mut thread_recv_buf.borrow_mut()[..];
        //         async_std::task::spawn(async move {
        //             loop {
        //                 let rr = 
        //                     this.0
        //                         .socket
        //                         .recv_package(
        //                             ProxyPacketDecode {
        //                                 proxy_datagram: this.0.proxy_datagram.as_ref(),
        //                             }
        //                         )
        //                         .await;
        //                 /*
        //                 let rr = self.0.socket.recv_from(recv_buf);
        //                 if rr.is_ok() {
        //                     let (len, from) = rr.unwrap();
        //                     let recv = &recv_buf[..len];
        //                     trace!("{} recv datagram len {} from {:?}", self, len, from);
        //                     self.on_proxied_datagram(recv, from);
        //                 } else {
        //                     let err = rr.err().unwrap();
        //                     if let Some(10054i32) = err.raw_os_error() {
        //                         // In Windows, if host A use UDP socket and call sendto() to send something to host B,
        //                         // but B doesn't bind any port so that B doesn't receive the message,
        //                         // and then host A call recvfrom() to receive some message,
        //                         // recvfrom() will failed, and WSAGetLastError() will return 10054.
        //                         // It's a bug of Windows.
        //                         trace!("{} socket recv failed for {}, ingore this error", self, err);
        //                     } else {
        //                         info!("{} socket recv failed for {}, break recv loop", self, err);
        //                         break;
        //                     }
        //                 }
        //                 */
        //             }
        //         })
        //     });
        // }
    }

    async fn on_proxied_datagram(&self, datagram: &[u8], from: Endpoint) {

        let mix_hash = 
            match Option::<KeyMixHash>::deserialize(datagram) {
                Ok((mix_hash, _)) => {
                    mix_hash
                }
                Err(err) => {
                    error!("failed KeyMixHash::deserialize from {from} with err {err}");
                    return;
                }
            };

        if let Some(mix_hash) = mix_hash {
            self.0.proxy_datagram.on_proxied_datagram(mix_hash, datagram, from).await;
        }

    }

}

impl ProxyInterface {
    pub async fn send_to(&self, datagram: &[u8], sockaddr: &Endpoint) {
        let _ = self.0.socket.send_data_to(datagram, sockaddr).await;
    }
}
