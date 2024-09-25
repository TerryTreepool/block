
use std::{sync::Arc, path::PathBuf, net::{SocketAddr, Ipv4Addr}};

use discovery_util::{*, protocol::{ParsePackage, v0::{SearchResp, Search}, BuildPackage, Head, Command}};
use log::{debug, warn};
use near_base::{NearResult, NearError, ErrorCode};
use near_core::get_service_path;

use common::RuntimeProcessTrait;

use crate::configures;

#[derive(Clone)]
pub(super) struct Config {
    #[allow(unused)]
    work_path: PathBuf,
}

struct ProcessComponents {
}

struct ProcessImpl {
    service_name: String,
    config: Config,

    components: Option<ProcessComponents>,
}

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

unsafe impl Send for Process {}
unsafe impl Sync for Process {}

impl Process {
    pub fn new(service_name: &str) -> NearResult<Box<Self>> {
        let config = Config {
            work_path: get_service_path(service_name),
        };

        let ret = Self(Arc::new(ProcessImpl{
            service_name: service_name.to_owned(),
            config: config.clone(),
            components: None,
        }));

        let mut_ret = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ProcessImpl) };
        mut_ret.components = Some(ProcessComponents {
        });

        Ok(Box::new(ret))
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn service_name(&self) -> &str {
        &self.0.service_name
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn config(&self) -> &Config {
        &self.0.config
    }

}

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {
    async fn run(&self) -> NearResult<()> {

        main_run().await?;

        async_std::task::block_on(async_std::future::pending::<()>());

        Ok(())
    }

    fn quit(&self) {
        
    }
}


pub struct UdpCast {
    fd: async_std::net::UdpSocket,
}

impl UdpCast {
    pub async fn bind(host: &str, port: u16) -> NearResult<Self> {
        let fd = 
            async_std::net::UdpSocket::bind(format!("{host}:{port}"))
                .await
                .map_err(| e | {
                    NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("failed bind {host}:{port} with err: {e}"))
                })?;

        let multicast_addr = GROUP_HOST.clone();
        let inter = Ipv4Addr::new(0, 0, 0, 0);
        fd.join_multicast_v4(multicast_addr.clone(), inter.clone())
            .map_err(| e | {
                NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("failed add-member-group with err: {e}"))
            })?;

        Ok(Self{
            fd,
        })
    }

    pub async fn recv_from(&self, text: &mut [u8]) -> NearResult<(usize, SocketAddr)> {
        match   self.fd
                    .recv_from(text)
                    .await {
            Ok((size, remote)) => {
                Ok((size, remote))
            }
            Err(err) => {
                if let Some(10054i32) = err.raw_os_error() {
                    Err(NearError::new(ErrorCode::NEAR_ERROR_RETRY, "retry"))
                    // In Windows, if host A use UDP socket and call sendto() to send something to host B,
                    // but B doesn't bind any port so that B doesn't receive the message,
                    // and then host A call recvfrom() to receive some message,
                    // recvfrom() will failed, and WSAGetLastError() will return 10054.
                    // It's a bug of Windows.
                    /* trace!("{} socket recv failed for {}, ignore this error", self, err); */
                } else {
                    // info!("{} socket recv failed for {}, break recv loop", self, err);
                    Err(NearError::from(err))
                }
            }
        }

    }
}

impl std::ops::Drop for UdpCast {
    fn drop(&mut self) {
        let multicast_addr = GROUP_HOST.clone();
        let inter = Ipv4Addr::new(0, 0, 0, 0);

        self.fd.leave_multicast_v4(multicast_addr, inter).unwrap();
    }
}

async fn recv_process(mut data: ParsePackage) -> NearResult<Vec<u8>> {
    let head = data.take_head();

    match &head.cmd {
        Command::Search => {
            search_process(head, data.take_body()).await
        }
        Command::SearchResp => {
            // ignore
            Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, "resp"))
        }
        Command::None => {
            // ignore
            Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, "none"))
        }
    }
}

async fn search_process(head: Head, _: Search) -> NearResult<Vec<u8>> {
    debug_assert_eq!(head.cmd as u8, Command::Search as u8);

    let data = 
        BuildPackage{
            head: Head{
                ver: head.ver,
                cmd: Command::SearchResp,
                uid: head.uid,
            },
            body: Box::new(SearchResp {
                desc: configures::Configures::get_instance().desc.clone(),
            }),
        }
        .build()?;

    Ok(data)
}

async fn main_run() -> NearResult<()> {
    let cast = Arc::new(UdpCast::bind("0.0.0.0", GROUP_PORT).await?);

    // configures init
    let _ = configures::Configures::get_instance();

    async_std::task::spawn(async move {
        loop {
            let mut text = [0u8; 2048];
            match cast.recv_from(&mut text).await {
                Ok((size, remote)) => {
                    debug!("data-size:{size}, remote:{remote}");
                    match ParsePackage::parse(&text[..size]) {
                        Ok(data) => {
                            let cast_clone = cast.clone();
                            async_std::task::spawn(async move {
                                match recv_process(data, ).await {
                                    Ok(data) => {
                                        let _ = cast_clone.fd.send_to(&data, remote).await;
                                    }
                                    Err(_e) => {
                                        // warn
                                        warn!("failed recv process with err: {_e}")
                                    }
                                }
                            });
                        }
                        Err(_e) => {
                            // warn
                            warn!("failed recv package with err: {_e}")
                        }
                    }
                }
                Err(e) if e.errno() == ErrorCode::NEAR_ERROR_RETRY => {
                    continue;
                }
                _ => { break; }
            }
        }
    });

    Ok(())
}
