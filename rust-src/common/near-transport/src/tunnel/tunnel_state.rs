use std::sync::{Arc, Mutex};

use log::{debug, error, trace};

use near_base::{
    any::AnyNamedObject, now, sequence::SequenceString, AesKey, ErrorCode, NearResult, ObjectId,
    Timestamp,
};

use crate::package::{
    AckAckTunnel, AckTunnel, AnyNamedRequest, Exchange, PackageBuilder, PackageHeader,
    PackageHeaderExt, SequenceBuild,
};

use super::{tunnel::State, TunnelStateTrait};

enum StateImpl {
    Connecting(ConnectingState),
    Activing(ActivingState),
    Established(EstablishedState),
    Dead,
}

struct ConnectingState;

struct ActivingState {
    timestamp: Timestamp, // If I'm client timestamp is local, else is remote
}

struct EstablishedState {
    timestamp: Timestamp,
}

struct TunnelStateImpl {
    state: StateImpl,
}

pub struct TunnelState {
    local: AnyNamedObject,
    state: Mutex<TunnelStateImpl>,
    aes_key: AesKey,
}

pub(super) struct TunnelExchangeData {
    #[allow(unused)]
    pub timestamp: Timestamp,
}
pub(super) type TunnelExchangeDataPtr = Arc<TunnelExchangeData>;

impl From<&EstablishedState> for TunnelExchangeData {
    fn from(state: &EstablishedState) -> Self {
        Self {
            timestamp: state.timestamp,
        }
    }
}

impl TunnelState {
    pub fn new(local: AnyNamedObject, aes_key: AesKey) -> Self {
        Self {
            local,
            state: Mutex::new(TunnelStateImpl {
                state: StateImpl::Connecting(ConnectingState {}),
            }),
            aes_key,
        }
    }
}

impl TunnelState {
    pub(super) fn active(
        &self,
        remote_id: &ObjectId,
    ) -> NearResult<(SequenceString, PackageBuilder)> {
        let tunnel_state = &mut *self.state.lock().unwrap();
        let state = &mut tunnel_state.state;
        match state {
            StateImpl::Connecting(_) | StateImpl::Dead | StateImpl::Established(_) => {
                let timestamp = now();

                *state = StateImpl::Activing(ActivingState {
                    timestamp: timestamp,
                });

                let seq = {
                    SequenceBuild {
                        requestor: self.local.object_id(),
                        now: timestamp,
                        sync_times: 0,
                    }
                }
                .build()?;

                Ok((
                    seq,
                    PackageBuilder::build_head(seq.clone(), None)
                        .build_topic(
                            None,
                            self.local.object_id().clone(),
                            remote_id.clone(),
                            None,
                        )
                        .build_body(AnyNamedRequest::with_exchange(Exchange {
                            aes_key: self.aes_key.clone(),
                            send_time: timestamp,
                            from_device: self.local.clone(),
                        })),
                ))
            }
            StateImpl::Activing(activing_state) => {
                let seq = {
                    SequenceBuild {
                        requestor: self.local.object_id(),
                        now: now(),
                        sync_times: 0,
                    }
                }
                .build()?;

                Ok((
                    seq,
                    PackageBuilder::build_head(seq.clone(), None)
                        .build_topic(
                            None,
                            self.local.object_id().clone(),
                            remote_id.clone(),
                            None,
                        )
                        .build_body(AnyNamedRequest::with_exchange(Exchange {
                            aes_key: self.aes_key.clone(),
                            send_time: activing_state.timestamp,
                            from_device: self.local.clone(),
                        })),
                ))
            } // StateImpl::Dead => {
              //     Err(NearError::new(ErrorCode::NEAR_ERROR_TUNNEL_CLOSED, "dead"))
              // }
              // _ => {
              //     Err(NearError::new(ErrorCode::NEAR_ERROR_ACTIVED, "actived"))
              // }
        }
    }
}

impl TunnelStateTrait<Exchange, TunnelExchangeDataPtr> for TunnelState {
    fn on_tunnel_event(
        &self,
        _head: PackageHeader,
        _headext: PackageHeaderExt,
        body: Exchange,
    ) -> State<TunnelExchangeDataPtr> {
        let tunnel_state = &mut *self.state.lock().unwrap();
        let state = &mut tunnel_state.state;

        match state {
            StateImpl::Connecting(_) => {
                *state = StateImpl::Activing(ActivingState {
                    timestamp: body.send_time,
                });
                State::Connecting
            }
            StateImpl::Activing(_) => State::Connecting,
            StateImpl::Established(data) => {
                State::Established(Arc::new(TunnelExchangeData::from(&*data)))
            }
            StateImpl::Dead => State::Dead,
        }
    }
}

impl TunnelStateTrait<AckTunnel, TunnelExchangeDataPtr> for TunnelState {
    fn on_tunnel_event(
        &self,
        _head: PackageHeader,
        head_ext: PackageHeaderExt,
        body: AckTunnel,
    ) -> State<TunnelExchangeDataPtr> {
        let tunnel_state = &mut *self.state.lock().unwrap();
        let state = &mut tunnel_state.state;

        match state {
            StateImpl::Activing(acting_state) => {
                trace!("on_tunnel_event: StateImpl::Activing");

                if body.result == ErrorCode::NEAR_ERROR_SUCCESS.into_u16() {
                    debug!(
                        "sucess exchange from remote:{} to local:{}",
                        head_ext, self.local
                    );
                    let established_state = EstablishedState {
                        timestamp: acting_state.timestamp,
                    };
                    let r =
                        State::Established(Arc::new(TunnelExchangeData::from(&established_state)));
                    *state = StateImpl::Established(established_state);
                    r
                } else {
                    error!("result is error, r = {}", body.result);
                    *state = StateImpl::Dead;
                    State::Dead
                    // Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, format!("failed exchange from remote:{} with err: {}", head_ext.from(), body.result)))
                }
            }
            StateImpl::Established(data) => {
                trace!("on_tunnel_event: StateImpl::Established");
                debug!(
                    "sucess exchange from remote:{} to local:{}",
                    head_ext, self.local
                );
                State::Established(Arc::new(TunnelExchangeData::from(&*data)))
            }
            StateImpl::Connecting(_) => {
                trace!("on_tunnel_event: StateImpl::Connecting");
                unreachable!()
            }
            StateImpl::Dead => {
                trace!("on_tunnel_event: StateImpl::Dead");
                State::Dead
            }
        }
    }
}

impl TunnelStateTrait<AckAckTunnel, TunnelExchangeDataPtr> for TunnelState {
    fn on_tunnel_event(
        &self,
        head: PackageHeader,
        _head_ext: PackageHeaderExt,
        _body: AckAckTunnel,
    ) -> State<TunnelExchangeDataPtr> {
        trace!("TunnelState::on_tunnel_event, head: {head}, AckAckTunnel: {_body}");

        let tunnel_state = &mut *self.state.lock().unwrap();
        let state = &mut tunnel_state.state;

        match state {
            StateImpl::Activing(acting_state) => {
                let established_state = EstablishedState {
                    timestamp: acting_state.timestamp,
                };
                let r = State::Established(Arc::new(TunnelExchangeData::from(&established_state)));
                *state = StateImpl::Established(established_state);
                // info!("handshake package from: {{{}}} on tunnel: {{{}}}, tunnel state: Activing => Established", head_ext.from(), self.interface);
                r
            }
            StateImpl::Established(established_state) => {
                // info!("handshake package from: {{{}}} on tunnel: {{{}}}, tunnel state was Established", head_ext.from(), self.interface);
                State::Established(Arc::new(TunnelExchangeData::from(&*established_state)))
            }
            StateImpl::Connecting(_) => unreachable!(),
            StateImpl::Dead => State::Dead,
            // info!("handshake package from: {{{}}}, tunnel state is error", head_ext.from());
            // Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, format!("failed exchange from remote:{}", head_ext.from())))
        }
    }
}

pub struct TunnelStateGuard(Arc<TunnelState>);

impl TunnelStateGuard {
    pub fn new(local: AnyNamedObject, aes_key: AesKey) -> Self {
        Self(Arc::new(TunnelState::new(local, aes_key)))
    }
}

impl std::ops::Deref for TunnelStateGuard {
    type Target = TunnelState;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

unsafe impl Sync for TunnelStateGuard {}
unsafe impl Send for TunnelStateGuard {}
