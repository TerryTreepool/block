
use std::{sync::Arc, time::Duration};

use crossbeam::epoch::Atomic;
use log::{trace, error};
use near_base::{EndpointPair, ObjectId, Endpoint, Timestamp, ErrorCode, DeviceObject, NearResult};

use crate::package::PackageDataSet;
use crate::{network::UdpInterface, Stack};

use super::DynamicTunnel;
use super::tunnel_state::{TunnelStateGuard, TunnelExchangeDataPtr};
use super::container::TunnelContainer;
use super::tunnel::Tunnel as TunnelTrait;


#[derive(Clone)]
pub struct Config {
    pub holepunch_interval: Duration,   // 200ms
    pub connect_timeout: Duration,      // 5s
    pub ping_interval: Duration,        // 30s 
    pub ping_timeout: Duration,         // 180s
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            holepunch_interval: Duration::from_millis(200),
            connect_timeout: Duration::from_secs(5),
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(60 * 3),
        }
    }
}

struct TunnelImpl {
    // proxy: ProxyType,
    // state: RwLock<TunnelState>,
    // keeper_count: AtomicI32,
    // last_active: AtomicU64,
    // retain_connect_timestamp: AtomicU64,
    // mtu: usize,

    owner: TunnelContainer,
    stack: Stack,
    remote_device_id: ObjectId,
    local_remote: EndpointPair,
    interface: UdpInterface,
    tunnel_sync_data: Atomic<Option<TunnelExchangeDataPtr>>,
    connect_state: TunnelStateGuard,

}

#[derive(Clone)]
pub struct Tunnel(Arc<TunnelImpl>);

impl std::fmt::Display for Tunnel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UdpTunnel{{remote_device:{}, local:{}, remote:{}}}", self.0.remote_device_id.to_string(), self.0.local_remote.local(), self.0.local_remote.remote())
    }
}

impl Tunnel {

    // pub(super) fn new(owner: TunnelContainer,
    //                 stack: WeakStack,
    //                 ep_pair: EndpointPair,
    //                 remote: ObjectId,
    //                 interface: TcpInterface) -> Self {

    pub(super) fn new(
        owner: TunnelContainer,
        stack: Stack,
        ep_pair: EndpointPair,
        remote: ObjectId,
        interface: UdpInterface,
    ) -> Self {
        Self(Arc::new(TunnelImpl{
            connect_state: TunnelStateGuard::new(stack.local().clone(), ),
            owner,
            stack: stack.clone(),
            remote_device_id: remote,
            local_remote: ep_pair,
            interface,
            tunnel_sync_data: Atomic::new(None),
        }))

        // let local = interface.local();
        // let state = TunnelState::Connecting(ConnectingState {
        //     container: container.clone(), 
        //     owner: owner.clone_as_tunnel_owner(), 
        //     interface, 
        //     waiter: StateWaiter::new()
        // });
        // let tunnel = Self(Arc::new(TunnelImpl {
        //     mtu: MTU,
        //     local, 
        //     remote, 
        //     proxy, 
        //     state: RwLock::new(state), 
        //     keeper_count: AtomicI32::new(0), 
        //     last_active: AtomicU64::new(0)
        // }));
        
        // {
        //     let tunnel = tunnel.clone();
        //     let connect_timeout = container.config().udp.connect_timeout;
        //     task::spawn(async move {
        //         match future::timeout(connect_timeout, tunnel.wait_active()).await {
        //             Ok(_state) => {
        //                 // assert_eq!(state, tunnel::TunnelState::Active, "state should be active");
        //             }, 
        //             Err(_err) => {
        //                 let waiter = {
        //                     let state = &mut *tunnel.0.state.write().unwrap();
        //                     match state {
        //                         TunnelState::Connecting(connecting) => {
        //                             let mut waiter = StateWaiter::new();
        //                             connecting.waiter.transfer_into(&mut waiter);
        //                             *state = TunnelState::Dead;
        //                             Some(waiter)
        //                         }, 
        //                         TunnelState::Active(_) => {
        //                             // do nothing
        //                             None
        //                         },
        //                         TunnelState::Dead => {
        //                             // do nothing
        //                             None
        //                         }
        //                     }
        //                 };
        //                 if let Some(waiter) = waiter  {
        //                     info!("{} dead for connecting timeout", tunnel);
        //                     waiter.wake();
        //                     owner.sync_tunnel_state(&DynamicTunnel::new(tunnel.clone()), tunnel::TunnelState::Connecting, tunnel::TunnelState::Dead);
        //                 }
        //             }
        //         }
        //     });
        // }

        // tunnel
    }

    pub fn config(&self) -> &Config {
        &self.0.stack.config().tunnel.udp_config
    }

    pub(super) fn active(&self, remote: &DeviceObject) {
        trace!("active: remote: {remote}");

        match self.0.connect_state.active(remote.object_id()) {
            Ok((sequence, builder)) => {
                let interface = self.0.interface.clone();
                let arc_self = self.clone();
                async_std::task::spawn(async move {
                    if let Ok(r) =
                        builder.build(None)
                            .await
                            .map_err(| err | {
                                error!("failed build EXCHANGE package to {} with {}", arc_self, err);
                                err
                            }) {
    
                        // let _ = interface.send_data(r.dataset(0).unwrap().as_ref())
                        //                 .await
                        //                 .map_err(| err | {
                        //                     error!("failed send data to {} with {}", interface, err);
                        //                     err
                        //                 });
                    }
                });
            }
            Err(err) => {
                match err.errno() {
                    ErrorCode::NEAR_ERROR_ACTIVED => {

                    },
                    ErrorCode::NEAR_ERROR_TUNNEL_CLOSED => {
                        // tunnel is dead
                    }
                    _ => {
                        error!("failed to active with {}", err);
                        // ignore
                    }
                }
            }
        }
    }

}



#[async_trait::async_trait]
impl TunnelTrait for Tunnel {
    fn clone_as_tunnel(&self) -> DynamicTunnel {
        DynamicTunnel::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn local(&self) -> &Endpoint {
        self.0.local_remote.local()
    }

    fn remote(&self) -> &Endpoint {
        self.0.local_remote.remote()
    }

    fn ptr_eq(&self, other: &DynamicTunnel) -> bool {
        let tunnel: &Tunnel = other.as_ref();
        Arc::ptr_eq(&self.0, &tunnel.0)
    }

    fn is_closed(&self) -> bool {
        // self.0.interface.is_closed()
        false
    }

    // fn send_package(&self, package: &DynamicPackage) -> NearResult<usize> {
    //     Ok(0)
    // }
    async fn send_package(&self, package: PackageDataSet) -> NearResult<()> {
        Ok(())
    }


    fn reset(&self) {

    }

    fn update_time(&self) -> Timestamp {
        0
    }
    
}

