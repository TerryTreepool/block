

use std::{net::TcpListener, sync::{atomic::AtomicBool, Arc, Mutex}};

use async_std::{net::{TcpListener as AsyncTcpListener, TcpStream as AsyncTcpStream}, task::JoinHandle};

use log::{info, error, trace, debug, };
use near_base::*;

use crate::network::{interface::PackageDecodeDef, TcpStateEventTrait};

use super::{data_context::NetInterface,
            TcpInterface, TcpPackageEventTrait, NetManager, Interface};

enum TcpState {
    Listener(ListenState),
    Connector(ConnectState),
}

struct ListenState {
    #[allow(unused)]
    local: Endpoint,
    listener: AsyncTcpListener,
}

// #[derive(Clone)]
struct ConnectState {
    // local_ep: Endpoint,
    #[allow(unused)]
    remote_ep: Endpoint,
    remote: DeviceObject,
    connector: TcpInterface,
}

struct TcpImpl {
    manager: NetManager,
    state: TcpState,
    running: AtomicBool,
    fut: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Clone)]
pub struct Tcp(Arc<TcpImpl>);

impl Tcp {
    pub fn bind(manager: NetManager, local: &Endpoint, ) -> NearResult<Self> {
        if !local.is_tcp() {
            unreachable!("must bind TCP protocol")
        }

        let listener = {
            if local.is_ipv6() {
                #[cfg(windows)]
                {
                    let default_local = local.clone();
                    TcpListener::bind(default_local)
                        .map_err(|err| {
                            NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, 
                                format!("failed bind() ipv6 with errno({})", err))

                        })
                }

                #[cfg(not(windows))]
                {
                    Err(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN_PROTOCOL, "don't support ipv6"))
                    // todo!()
                }
            } else {    // ipv4
                let default_local = local.clone();
                TcpListener::bind(default_local)
                    .map_err(|err| {
                        NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, 
                            format!("failed bind() ipv4 with errno({})", err))
                    })
            }
        }?;

        Ok(Self(Arc::new(TcpImpl{
            manager,
            state: TcpState::Listener(ListenState {
                local: local.clone(),
                listener: AsyncTcpListener::from(listener),
            }),
            running: AtomicBool::new(true),
            fut: Mutex::new(None),
        })))
    }

    pub async fn connect(manager: NetManager, remote_ep: Endpoint, remote: DeviceObject) -> NearResult<Self> {
        if !remote_ep.is_tcp() {
            unreachable!("must connect TCP protocol")
        }

        let connector = 
            TcpInterface::connect(&remote_ep, manager.stack().config().tunnel.container.tcp.connect_timeout)
                .await?;

        Ok(Self(Arc::new(TcpImpl{
            manager,
            state: TcpState::Connector(ConnectState {
                remote_ep,
                remote,
                connector
            }),
            running: AtomicBool::new(true),
            fut: Mutex::new(None),
        })))
    }

}

impl NetInterface for Tcp {
    fn clone_as_interface(&self) -> Box<dyn NetInterface> {
        Box::new(self.clone())
    }

    fn clone_into_interface(&self) -> Box<dyn Interface> {
        match &self.0.state {
            TcpState::Connector(connector) => connector.connector.clone_as_interface(),
            TcpState::Listener(_listener) => unimplemented!(),
        }
    }

    fn start(&self) -> NearResult<()> {

        let mut_fut = &mut *self.0.fut.lock().unwrap();

        if let None = mut_fut {
            let tcp = self.clone();
            *mut_fut = Some(
                async_std::task::spawn(async move {
                    match &tcp.0.state {
                        TcpState::Listener(listener) => {
                            trace!("startup listen...");
                            // let async_listener = listener
                            Tcp::listen_proc(&tcp, &listener.listener).await;
                        }
                        TcpState::Connector(connector) => {
                            trace!("startup connect proc ...");
                            tcp.0.manager.stack().on_connected(connector.connector.clone(), &connector.remote);

                            tcp.recv_progress(connector.connector.clone())
                                .await;
                        }
                    }
                }));
        }

        Ok(())
    }

}

impl Tcp {
    async fn listen_proc(tcp: &Tcp, listener: &AsyncTcpListener) {
        use std::io::ErrorKind;

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let me = tcp.clone();
                    async_std::task::spawn( async move {
                        if let Ok(interface) = TcpInterface::accept(AsyncTcpStream::from(stream)).await {
                            me.recv_progress(interface)
                                .await;
                        }
                    });
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::Interrupted | 
                        ErrorKind::WouldBlock | 
                        ErrorKind::AlreadyExists | 
                        ErrorKind::TimedOut => continue,
                        _ => {
                            // warn!("tcp-listener accept fatal error({}). will stop.", e);
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn recv_progress(&self, interface: TcpInterface) {
        let interface = interface.clone();

        while self.0.running.load(std::sync::atomic::Ordering::SeqCst) {
            match interface.recv_package(PackageDecodeDef).await {
                Ok(package) => {
                    debug!("{package} from {interface}");
                    if let Err(e) = 
                        self.0
                            .manager
                            .stack()
                            .on_tcp_package(interface.clone(), package)
                            .await {
                        error!("on_tcp_package error: {e}");
                    }
                }
                Err(err) => {
                    match err.into_errno().into() {
                        ErrorCode::NEAR_ERROR_RETRY => continue,
                        _ => {
                            error!("{} failed to recv_package with error: {}", interface, err);
                            break;
                        }
                    }
                }
            }
        }

        info!("{} has closed.", interface);
        interface.close();

        match &self.0.state {
            TcpState::Connector(c) => {
                self.0.manager.on_closed(&interface);
                self.0.manager.stack().on_closed(&interface, c.remote.object_id());
            }
            _ => {
                self.0.manager.on_closed(&interface);
            }
        }
    }

}
