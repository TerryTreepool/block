use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use log::{error, trace, warn};

use near_base::{DeviceObject, Endpoint, ErrorCode, NearError, NearResult};

use crate::Stack;

use super::data_context::NetInterface;
use super::{Tcp, TcpStateEventTrait, Udp};

struct NetManagerImpl {
    stack: Stack,
    tcp: RwLock<BTreeMap<Endpoint, Tcp>>,
    udp: RwLock<BTreeMap<Endpoint, Udp>>,
}

#[derive(Clone)]
pub struct NetManager(Arc<NetManagerImpl>);

impl NetManager {
    fn new(stack: Stack) -> Self {
        NetManager(Arc::new(NetManagerImpl {
            stack,
            tcp: RwLock::new(BTreeMap::new()),
            udp: RwLock::new(BTreeMap::new()),
        }))
    }

    pub async fn listen(stack: Stack, endpoints: &[Endpoint]) -> NearResult<Self> {
        let ret = NetManager::new(stack);

        for e in endpoints {
            let _ = ret.bind_interface(e).await.map_err(|err| {
                let error_string = format!("failed bind {e} with err={err}");
                warn!("{error_string}");
                err
            });
        }

        Ok(ret)
    }

    pub async fn connect_tcp(
        stack: Stack,
        endpoint: &Endpoint,
        remote: &DeviceObject,
    ) -> NearResult<Self> {
        trace!("connect: target: {:?}", endpoint);

        let ret = NetManager::new(stack);

        ret.connect_tcp_interface(endpoint, remote).await?;

        Ok(ret)
    }

    pub fn stack(&self) -> Stack {
        self.0.stack.clone()
    }

    #[allow(unused)]
    pub fn udp(&self) -> Vec<Udp> {
        self.0.udp.read().unwrap()
            .values()
            .into_iter()
            .map(| iter | iter.clone())
            .collect()
    }

    #[allow(unused)]
    pub fn tcp(&self) -> Vec<Tcp> {
        self.0.tcp.read().unwrap()
            .values()
            .into_iter()
            .map(| iter | iter.clone())
            .collect()
    }

}

impl NetManager {
    async fn bind_interface(&self, endpoint: &Endpoint) -> NearResult<()> {

        if endpoint.is_tcp() {
            self.bind_tcp_interface(endpoint).await?;
            Ok(())
        } else if endpoint.is_udp() {
            self.bind_udp_interface(endpoint).await?;
            Ok(())
        } else {
            Err(NearError::new(
                ErrorCode::NEAR_ERROR_UNKNOWN_PROTOCOL,
                "unknown protocol",
            ))
        }

    }

    async fn bind_tcp_interface(&self, endpoint: &Endpoint) -> NearResult<Tcp> {
        if !endpoint.is_tcp() {
            unreachable!("must TCP endpoint");
        }

        if self.0.tcp.read().unwrap().get(endpoint).is_some() {
            Err(NearError::new(ErrorCode::NEAR_ERROR_ALREADY_EXIST, "[{endpoint}] has been bind."))
        } else {
            Ok(())
        }?;

        let tcp = 
            Tcp::bind(self.clone(), endpoint)
                .map_err(| e | {
                    error!("failed bind [{endpoint}] with err: {e}");
                    e
                })?;

        {
            self.0.tcp.write().unwrap()
                .entry(endpoint.clone())
                .or_insert(tcp.clone());
        };

        match tcp.start() {
            Ok(_) => { Ok(tcp) }
            Err(err) => {
                self.0.tcp.write().unwrap().remove(endpoint);
                return Err(err);
            }
        }
    }

    pub async fn bind_udp_interface(&self, endpoint: &Endpoint) -> NearResult<Udp> {
        if !endpoint.is_udp() {
            unreachable!("must UDP endpoint");
        }

        if self.0.udp.read().unwrap().get(endpoint).is_some() {
            Err(NearError::new(ErrorCode::NEAR_ERROR_ALREADY_EXIST, format!("[{endpoint}] has been bind.")))
        } else {
            Ok(())
        }?;

        let udp = 
            Udp::bind(self.clone(), endpoint.clone())
                .map_err(| e | {
                    error!("failed bind [{endpoint}] with err: {e}");
                    e
                })?;

        {
            self.0.udp.write().unwrap()
                .entry(endpoint.clone())
                .or_insert(udp.clone());
        };

        match udp.start() {
            Ok(_) => { Ok(udp) }
            Err(err) => {
                self.0.udp.write().unwrap().remove(endpoint);
                return Err(err);
            }
        }
    }

    pub async fn connect_tcp_interface(
        &self,
        endpoint: &Endpoint,
        remote: &DeviceObject,
    ) -> NearResult<Box<dyn NetInterface>> {
        if !endpoint.is_tcp() {
            unreachable!("must TCP endpoint");
        }

        {
            if let Some(tcp) = self.0.tcp.read().unwrap().get(&endpoint) {
                return Ok(tcp.clone_as_interface());
            }
        }

        let tcp = {
            Tcp::connect(self.clone(), endpoint.clone(), remote.clone())
                .await
                .map_err(|e| {
                    error!("failed connect [{endpoint}] with err: {e}");
                    e
                })
        }?;

        {
            self.0.tcp.write().unwrap()
                .entry(endpoint.clone())
                .or_insert(tcp.clone());
        };

        match tcp.start() {
            Ok(_) => {}
            Err(err) => {
                self.0.tcp.write().unwrap().remove(endpoint);
                return Err(err);
            }
        }

        Ok(tcp.clone_as_interface())
    }

    // async fn connect_udp_interface(
    //     &self,
    //     remote_endpoint: &Endpoint,
    //     remote: &DeviceObject,
    // ) -> NearResult<Box<dyn NetInterface>> {
        
    //     if !remote_endpoint.is_udp() {
    //         unreachable!("must UDP endpoint");
    //     }

    //     {
    //         if let Some(udp) = self.0.udp.read().unwrap().get(&remote_endpoint) {
    //             return Ok(udp.clone_as_interface());
    //         }
    //     }

    //     let udp = {
    //         Udp::connect(self.clone(), remote_endpoint, remote.clone())
    //             .await
    //             .map(|udp| udp.clone_as_interface())
    //             .map_err(|e| {
    //                 error!("failed connect [{remote_endpoint}] with err: {e}");
    //                 e
    //             })
    //     }?;

    //     {
    //         self.0.udp.write().unwrap()
    //             .entry(remote_endpoint.clone())
    //             .or_insert(udp.clone_as_interface());
    //     };

    //     match udp.start() {
    //         Ok(_) => {}
    //         Err(err) => {
    //             self.0.udp.write().unwrap().remove(remote_endpoint);
    //             return Err(err);
    //         }
    //     }

    //     Ok(udp.clone_as_interface())
    // }

    // pub async fn connect_interface(
    //     &self,
    //     remote_endpoint: &Endpoint,
    //     remote: &DeviceObject,
    // ) -> NearResult<Box<dyn NetInterface>> {

    //     if remote_endpoint.is_tcp() {
    //         self.connect_tcp_interface(remote_endpoint, remote).await
    //     } else if remote_endpoint.is_udp() {
    //         self.connect_udp_interface(remote_endpoint, remote).await
    //     } else {
    //         Err(NearError::new(
    //             ErrorCode::NEAR_ERROR_UNKNOWN_PROTOCOL,
    //             "unknown protocol",
    //         ))
    //     }
    // }
}

impl TcpStateEventTrait for NetManager {
    fn on_closed(&self, interface: &super::TcpInterface) {
        self.0
            .tcp
            .write()
            .unwrap()
            .remove(interface.remote());
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test1() {
        use crossbeam::channel::{select, unbounded};
        use std::thread;
        use std::time::Duration;

        let (_s1, r1) = unbounded::<i32>();
        let (s2, r2) = unbounded();

        let s21 = s2.clone();
        let s22 = s2.clone();

        let j1 = thread::spawn(move || {
            for i in 0..100 {
                s21.send(i).unwrap();
                thread::sleep(Duration::from_millis(100));
            }
        });
        let j2 = thread::spawn(move || {
            for i in 100..200 {
                s22.send(i).unwrap();
                thread::sleep(Duration::from_millis(100));
            }
        });

        let r = std::thread::spawn(move || loop {
            select! {
                recv(r1) -> msg => if let Ok(msg) = msg { println!("{},", msg); },
                recv(r2) -> msg => if let Ok(msg) = msg { println!("{},", msg); },
                // default(Duration::from_millis(10)) => {
                //     println!("-");

                // },
            }
        });

        let _ = j1.join();
        let _ = j2.join();
        let _ = r.join();
    }
}
