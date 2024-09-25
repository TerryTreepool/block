
use std::{sync::{Arc, Mutex},
          time::Duration
    };

use near_core::{debug, trace, info, warn, error};

use near_base::*;

use crate::{Stack,
            package::{PackageHeader, PackageHeaderExt, Exchange, Ack, AckAck, PackageBuilder},
            network::{TcpInterface, MTU, DynamicInterface, }, tunnel::OnTunnelEventTrait,
    };

use super::{tunnel::{Tunnel as TunnelTrait, DynamicTunnel, TunnelState, TunnelStateTrait},
    };

#[derive(Clone)]
pub struct Config {
    pub connect_timeout: Duration,
    // pub confirm_timeout: Duration,
    // pub accept_timeout: Duration,
    // // 调用retain_keeper 之后延迟多久开始尝试从preactive 进入 active状态
    // pub retain_connect_delay: Duration,
    // pub ping_interval: Duration,
    // pub ping_timeout: Duration,

    // pub package_buffer: usize,
    // pub piece_buffer: usize,
    // // 检查发送piece buffer的间隔
    // pub piece_interval: Duration,
}

enum TunnelStateImpl {
    Connecting(ConnectingState),
    Activing(ActivingState),
    Established(EstablishedState),
    Dead,
}

struct ConnectingState {
    interface: TcpInterface,
}

struct ActivingState {
    interface: TcpInterface,
    remote_timestamp: Timestamp,
    syn_req: Sequence,
    aes_key: AesKey,
}

struct EstablishedState {
    interface: TcpInterface,
    remote_timestamp: Timestamp,
    syn_req: Sequence,
    aes_key: AesKey,
    dead_waiters: StateWaiter
}

struct TunnelImpl {
    remote_device_id: ObjectId,
    local_remote: EndpointPair,
    // keeper_count: AtomicI32,
    // last_active: AtomicU64,
    // retain_connect_timestamp: AtomicU64,
    state: Mutex<TunnelStateImpl>
}

#[derive(Clone)]
pub struct Tunnel(Arc<TunnelImpl>);

impl std::fmt::Display for Tunnel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TcpTunnel{{remote_device:{}, local:{}, remote:{}}}", self.0.remote_device_id.to_string(), self.0.local_remote.local(), self.0.local_remote.remote())
    }
}

impl Tunnel {
    pub(super) fn new(ep_pair: EndpointPair,
                      remote: ObjectId,
                      interface: TcpInterface) -> Self {
        Self(Arc::new(TunnelImpl{
            remote_device_id: remote,
            local_remote: ep_pair,
            state: Mutex::new(TunnelStateImpl::Connecting(ConnectingState{
                interface: interface
            }))
        }))
    }

    pub(super) fn active(&self, stack: Stack) {
        enum NextStep {
            Exchange(TcpInterface, PackageBuilder<Exchange>),
            None,
        }

        let next_step = {
            let state = &mut *self.0.state.lock().unwrap();
            match state {
                TunnelStateImpl::Connecting(connecting) => {
                    let activing_state = ActivingState {
                        interface: connecting.interface.clone(),
                        remote_timestamp: now(),
                        syn_req: Sequence::random(),
                        aes_key: AesKey::generate(),
                    };

                    let r = 
                        NextStep::Exchange(connecting.interface.clone(),
                                            PackageBuilder::build_head(activing_state.syn_req.generate())
                                                .build_topic(stack.local_device_id().clone(), None, None)
                                                .build_body(Exchange {
                                                    aes_key: activing_state.aes_key.clone(),
                                                    send_time: now(),
                                                    from_device: stack.local()
                                                })
                                          );
                        // NextStep::Exchange(connecting.interface.clone(),
                        //                     PackageBuilder::build_head(stack.local_device_id().clone(), None, activing_state.syn_req.generate())
                        //                         .build_body(Exchange {
                        //                             aes_key: activing_state.aes_key.clone(),
                        //                             send_time: now(),
                        //                             from_device: stack.local()
                        //                         })
                        //                   );
                        *state = TunnelStateImpl::Activing(activing_state);
                    r
                }
                _ => { 
                    info!("{} has actived", self); 
                    NextStep::None
                }
            }
        };

        match next_step {
            NextStep::Exchange(interface, builder) => {
                async_std::task::spawn(async move {
                    let mut package_data = [0u8; MTU];
                    if let Ok(_) = builder.build(&mut package_data, None)
                        .await
                        .map_err(| err | {
                            error!("failed build EXCHANGE package to {} with {}", interface, err); 
                            err
                        }) {
                        let _ = interface.send_data(&package_data)
                            .await
                            .map_err(| err | {
                                error!("failed send data to {} with {}", interface, err);
                                err
                            });
                    }
                });
            }
            _ => {},
        }
    }
}

impl TunnelStateTrait<Exchange, Stack> for Tunnel {
    fn on_tunnel_event(&self, head: &PackageHeader, headext: &PackageHeaderExt, body: &Exchange, stack: Stack) -> NearResult<()> {
        enum NextStep {
            Response(TcpInterface, ErrorCode),
            Ignore,
        }

        let next_step = {
            let state = &mut *self.0.state.lock().unwrap();
            match state {
                TunnelStateImpl::Connecting(connecting_state) => {
                    let interface = connecting_state.interface.clone();
                    *state = TunnelStateImpl::Activing(ActivingState{
                        syn_req: Sequence::from(head.sequence()),
                        aes_key: body.aes_key,
                        remote_timestamp: body.send_time,
                        interface: interface.clone(),
                    });

                    NextStep::Response(interface, ErrorCode::NEAR_ERROR_SUCCESS)
                }
                TunnelStateImpl::Activing(activing_state) => {
                    activing_state.syn_req = Sequence::from(head.sequence());
                    activing_state.aes_key = body.aes_key;
                    activing_state.remote_timestamp = body.send_time;
                    NextStep::Response(activing_state.interface.clone(), ErrorCode::NEAR_ERROR_SUCCESS)
                }
                TunnelStateImpl::Established(established_state) => {
                    established_state.syn_req = Sequence::from(head.sequence());
                    established_state.aes_key = body.aes_key.clone();
                    established_state.remote_timestamp = body.send_time;
                    NextStep::Response(established_state.interface.clone(), ErrorCode::NEAR_ERROR_SUCCESS)
                }
                TunnelStateImpl::Dead => {
                    NextStep::Ignore
                }
            }
        };

        // match ret {
        //     ErrorCode::NEAR_ERROR_SUCCESS =>
        //     _ =>
        //         error!("faiiled handshake package from: {{{}}} on tunnel: {{{}}}, with error: {}", head.from().to_string(), interface, ret),
        // }

        match next_step {
            NextStep::Response(interface, ret) => {
                let from = stack.local_device_id().clone();
                let seq = head.sequence().clone();
                async_std::task::spawn(async move {
                    match ret {
                        ErrorCode::NEAR_ERROR_SUCCESS =>
                            info!("handshake package from: {{{}}} on tunnel: {{{}}}, tunnel state: Connecting => Activing", from.to_string(), interface),
                        _ =>
                            error!("faiiled handshake package from: {{{}}} on tunnel: {{{}}}, with error: {}", from.to_string(), interface, ret),
                    }

                    let mut package_data = [0u8; MTU];
                    if let Ok((_package, _)) =
                        PackageBuilder::build_head(seq)
                            .build_topic(from, None, None)
                            .build_body(Ack{result: ret.into(), send_time: now()})
                            .build(&mut package_data, None)
                            .await
                            .map_err(| err | {
                                error!("failed build ACK package to {} with {}", interface, err);
                                err
                            }) {
                        // PackageBuilder::build_head(from, None, seq)
                        //     .build_body(Ack{result: ret.into(), send_time: now()})
                        //     .build(&mut package_data, None)
                        //     .await
                        //     .map_err(| err | {
                        //         error!("failed build ACK package to {} with {}", interface, err);
                        //         err
                        //     }) {
                        let _ = interface.send_data(&package_data)
                            .await
                            .map(|_| debug!("succeed send ACK pacage to {}", interface) )
                            .map_err(| err | error!("failed send data to {} with {}", interface, err) );
                    }
                });
            }
            _ => {}
        }

        Ok(())
    }
}

impl TunnelStateTrait<Ack, Stack> for Tunnel {
    fn on_tunnel_event(&self, head: &PackageHeader, head_ext: &PackageHeaderExt, body: &Ack, stack: Stack) -> NearResult<()> {
        enum NextStep {
            AckAck(TcpInterface, bool /* frist active */),
            Dead,
        }

        let next_step = {
            let state = &mut *self.0.state.lock().unwrap();
            match state {
                TunnelStateImpl::Activing(acting_state) => {
                    if body.result == ErrorCode::NEAR_ERROR_SUCCESS.into() {
                        let interface = acting_state.interface.clone();
                        debug!("sucess exchange from remote:{} to local:{}", head_ext.from(), stack.local_device_id().to_string());
                        *state = TunnelStateImpl::Established(EstablishedState{
                            interface: interface.clone(),
                            remote_timestamp: acting_state.remote_timestamp,
                            syn_req: acting_state.syn_req.clone(),
                            aes_key: acting_state.aes_key,
                            dead_waiters: StateWaiter::new(),
                        });
                        NextStep::AckAck(interface, true)
                    } else {
                        *state = TunnelStateImpl::Dead;
                        info!("failed exchange from remote:{} with err: {}", head_ext.from(), body.result);
                        NextStep::Dead
                    }
                }
                TunnelStateImpl::Established(established_state) => {
                    debug!("sucess exchange from remote:{} to local:{}", head_ext.from(), stack.local_device_id().to_string());
                    NextStep::AckAck(established_state.interface.clone(), false)
                }
                _ => unreachable!()
            }
        };

        match next_step {
            NextStep::AckAck(interface, first_active) => {
                info!("handshake package from: {{{}}} on tunnel: {{{}}}, tunnel state: Activing => Established", head_ext.from(), interface);
                let from = stack.local_device_id().clone();
                let seq = head.sequence().clone();
                let interface_c = interface.clone();
                async_std::task::spawn(async move {
                    let mut package_data = [0u8; MTU];
                    if let Ok((_package, _)) =
                        // PackageBuilder::build_head(from, None, seq)
                        //     .build_body(AckAck{ send_time: now() })
                        //     .build(&mut package_data, None)
                        //     .await
                        //     .map_err(| err | {
                        //         error!("failed build ACKACK package to {} with {}", interface, err);
                        //         err
                        //     }) {
                        PackageBuilder::build_head(seq)
                            .build_topic(from, None, None)
                            .build_body(AckAck{ send_time: now() })
                            .build(&mut package_data, None)
                            .await
                            .map_err(| err | {
                                error!("failed build ACKACK package to {} with {}", interface, err);
                                err
                            }) {
                        let _ = interface.send_data(&package_data)
                            .await
                            .map(|_| debug!("succeed send ACKACK pacage to {}", interface) )
                            .map_err(| err | error!("failed send data to {} with {}", interface, err) );
                    }
                });

                if first_active {
                    stack.on_actived(DynamicTunnel::new(self.clone()));
                    // stack.on_actived(self.clone());
                }
            }
            _ => {
                info!("failed handshake package from: {{{}}}, refused", head_ext.from());
            }
        }

        Ok(())
    }
}

impl TunnelStateTrait<AckAck, Stack> for Tunnel {
    fn on_tunnel_event(&self, _: &PackageHeader, head_ext: &PackageHeaderExt, _body: &AckAck, _: Stack) -> NearResult<()> {
        let state = &mut *self.0.state.lock().unwrap();
        match state {
            TunnelStateImpl::Activing(acting_state) => {
                let interface = acting_state.interface.clone();
                *state = TunnelStateImpl::Established(EstablishedState{
                    interface: interface.clone(),
                    remote_timestamp: acting_state.remote_timestamp,
                    syn_req: acting_state.syn_req.clone(),
                    aes_key: acting_state.aes_key,
                    dead_waiters: StateWaiter::new(),
                });
                info!("handshake package from: {{{}}} on tunnel: {{{}}}, tunnel state: Activing => Established", head_ext.from(), interface);
            }
            TunnelStateImpl::Established(established_state) => {
                info!("handshake package from: {{{}}} on tunnel: {{{}}}, tunnel state was Established", head_ext.from(), established_state.interface);
            }
            _ => {
                *state = TunnelStateImpl::Dead;
                info!("handshake package from: {{{}}}, tunnel state is error", head_ext.from());
            }
        }

        Ok(())
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

    fn state(&self) -> TunnelState {
        match &*self.0.state.lock().unwrap() {
            TunnelStateImpl::Connecting(_) | TunnelStateImpl::Activing(_) => TunnelState::Connecting,
            TunnelStateImpl::Established(state) => TunnelState::Established(state.remote_timestamp),
            _ => TunnelState::Dead,
        }
    }

    fn send_raw_data(&self, data: &[u8]) -> NearResult<usize> {
        Ok(0)
    }

    fn reset(&self) {

    }
}

