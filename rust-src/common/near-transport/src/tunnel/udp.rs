use std::{sync::Arc, time::Duration};

use crossbeam::epoch::{self as epoch, Atomic, Owned};

use log::{debug, error, info, trace};

use near_base::device::DeviceId;
use near_base::{sequence::SequenceString, *};

use crate::h::OnBuildPackage;
use crate::network::{Interface, UdpInterface, DataContext};
use crate::package::{
    AckAckTunnel, AckTunnel, AnyNamedRequest, Exchange, PackageDataSet,
    PackageHeader, PackageHeaderExt,
};
use crate::stack::BuildPackageV1;
use crate::tunnel::container::MessageEventTrait;
use crate::Stack;

use super::PostMessageTrait;
use super::{
    container::TunnelContainer,
    tunnel::{DynamicTunnel, State, Tunnel as TunnelTrait, TunnelStateTrait},
    tunnel_state::{TunnelExchangeDataPtr, TunnelStateGuard},
};

#[derive(Clone)]
pub struct Config {
    pub holepunch_interval: Duration,   // 200ms
    pub connect_timeout: Duration,
    pub send_timeout: Duration,
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

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            holepunch_interval: Duration::from_millis(200),
            connect_timeout: Duration::from_secs(5),
            send_timeout: Duration::from_secs(10),
        }
    }
}

// #[repr(u8)]
// enum TunnelStateImpl {
//     Connecting,
//     Actived(TunnelExchangeDataPtr),
//     Dead,
// }

struct TunnelImpl {
    owner: TunnelContainer,
    stack: Stack,
    remote_device_id: ObjectId,
    local_remote: EndpointPair,
    interface: UdpInterface,
    tunnel_sync_data: Atomic<Option<TunnelExchangeDataPtr>>,
    // keeper_count: AtomicI32,
    // last_active: AtomicU64,
    // retain_connect_timestamp: AtomicU64,
    connect_state: TunnelStateGuard,
}

#[derive(Clone)]
pub struct Tunnel(Arc<TunnelImpl>);

impl std::fmt::Display for Tunnel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UdpTunnel{{remote_device:{}, local:{}, remote:{}}}",
            self.0.remote_device_id.to_string(),
            self.0.local_remote.local(),
            self.0.local_remote.remote()
        )
    }
}

impl Tunnel {
    pub(super) fn new(
        owner: TunnelContainer,
        stack: Stack,
        ep_pair: EndpointPair,
        remote: ObjectId,
        interface: UdpInterface,
    ) -> Self {
        Self(Arc::new(TunnelImpl {
            connect_state: TunnelStateGuard::new(stack.local().clone(), stack.aes_key()),
            owner,
            stack: stack.clone(),
            remote_device_id: remote,
            local_remote: ep_pair,
            interface: interface.clone(),
            tunnel_sync_data: Atomic::new(None),
        }))
    }

    pub(super) fn active(&self, remote: &DeviceObject) {
        trace!("active: remote: {}", remote.object_id());

        match self.0.connect_state.active(remote.object_id()) {
            Ok((sequence, builder)) => {
                let arc_self = self.clone();
                async_std::task::spawn(async move {
                    if let Ok(package) = builder.build(None).await.map_err(|err| {
                        error!("failed build EXCHANGE package to {}:{} with {}", arc_self.peer_id(), arc_self.remote(), err);
                        err
                    }) {
                        info!("successfully build EXCHANGE to {}:{} sequence {:?}", arc_self.peer_id(), arc_self.remote(), sequence);

                        let _ = 
                            arc_self
                                .post_message((sequence, package.clone()))
                                .await
                                .map(| _ | {});
                    }
                });
            }
            Err(err) => {
                match err.errno() {
                    ErrorCode::NEAR_ERROR_ACTIVED => {}
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

    #[allow(unused)]
    pub(self) fn as_stack(&self) -> &Stack {
        &self.0.stack
    }

    #[allow(unused)]
    pub(self) fn tunnel_sync_data(&self) -> TunnelExchangeDataPtr {
        let guard = &epoch::pin();
        let curr = self
            .0
            .tunnel_sync_data
            .load(std::sync::atomic::Ordering::SeqCst, guard);

        match unsafe { curr.as_ref().unwrap() } {
            Some(tunnel_sync_data) => tunnel_sync_data.clone(),
            None => unreachable!(),
        }
    }

    pub(self) fn sync_tunnel_state(&self, state: State<TunnelExchangeDataPtr>) -> State {
        let state = {
            match state {
                State::Connecting => State::Connecting,
                State::Dead => State::Dead,
                State::Established(exchange_data) => {
                    let guard = &epoch::pin();
                    let curr = 
                        self
                            .0
                            .tunnel_sync_data
                            .load(std::sync::atomic::Ordering::SeqCst, guard);
                    match unsafe { curr.as_ref().unwrap() } {
                        None => {
                            let _ = 
                                self.0.tunnel_sync_data.compare_exchange(
                                    curr,
                                    Owned::new(Some(exchange_data)),
                                    std::sync::atomic::Ordering::SeqCst,
                                    std::sync::atomic::Ordering::SeqCst,
                                    guard,
                                );
                        }
                        _ => { /* ignore */ }
                    }
                    State::Established(())
                }
            }
        };

        // sync tunnel state into container
        self.0
            .owner
            .sync_tunnel_state(DynamicTunnel::new(self.clone()), &state);

        state
    }
}

// #[async_trait::async_trait]
// impl PostMessageTrait<(HeaderMeta, Data)> for Tunnel {
//     async fn post_message(&self, context: (HeaderMeta, Data)) -> NearResult<()> {
//         let (header_meta, body) = context;
//         let stack = self.as_stack();

//         match stack.build_package((header_meta, body)).await {
//             Ok((sequence, package)) => {
//                 trace!("send package: {sequence}-{package}");
//                 self.post_message((sequence, package)).await
//             }
//             Err(e) => Err(e),
//         }
//     }
// }

#[async_trait::async_trait]
impl PostMessageTrait<(SequenceString, Vec<DataContext>)> for Tunnel {

    type R = ();

    async fn post_message(
        &self, 
        context: (SequenceString, Vec<DataContext>)
    ) -> NearResult<Self::R> {
        let (sequence, data_context) = context;

        let package: PackageDataSet = data_context.try_into()?;

        self.post_message((sequence, package)).await
    }
}

#[async_trait::async_trait]
impl PostMessageTrait<(SequenceString, PackageDataSet)> for Tunnel {

    type R = ();

    async fn post_message(
        &self, 
        context: (SequenceString, PackageDataSet)
    ) -> NearResult<Self::R> {
        let (sequence, package) = context;

        if let Err(e) = self.send_package(package.clone()).await {
            self.0.owner.on_failure(self.clone_as_tunnel(), sequence, package, e.clone());
            Err(e)
        } else {
            self.0.owner.on_succeed(self.clone_as_tunnel(), sequence, package);
            Ok(())
        }
    }
}

impl Tunnel {
    // pub(super) async fn post_data(
    //     &self, 
    //     header_meta: HeaderMeta, 
    //     body: Data
    // ) -> NearResult<()> {
    //     let stack = self.as_stack();

    //     match stack.build_package((header_meta, body)).await {
    //         Ok((sequence, package)) => {
    //             trace!("send package: {sequence}-{package}");
    //             self.post_message_p(sequence, package).await
    //         }
    //         Err(e) => Err(e),
    //     }
    // }

    // pub(super) async fn post_data_context(
    //     &self, 
    //     sequence: SequenceString, 
    //     data_context: Vec<DataContext>
    // ) -> NearResult<()> {
    //     self.post_message_p(sequence, data_context.try_into()?).await
    // }

    // pub(self) async fn post_message_p(
    //     &self,
    //     sequence: SequenceString,
    //     package: PackageDataSet,
    // ) -> NearResult<()> {
    //     if let Err(e) = self.send_package(package.clone()).await {
    //         self.0.owner.on_failure(sequence, package, e.clone());
    //         Err(e)
    //     } else {
    //         self.0.owner.on_succeed(sequence, package);
    //         Ok(())
    //     }
    // }
    pub(super) async fn send_package(
        &self, 
        package: PackageDataSet
    ) -> NearResult<()> {

        for i in 0..package.dataset_count() {
            if let Some(data) = package.dataset(i) {
                self.0.interface.send_data_to(data.as_ref(), self.remote()).await?;
            } else {
                return Err(NearError::new(
                    ErrorCode::NEAR_ERROR_EXCEPTION,
                    "unreachable",
                ));
            }
        }

        Ok(())
    }


}

impl TunnelStateTrait<Exchange> for Tunnel {
    fn on_tunnel_event(
        &self,
        head: PackageHeader,
        headext: PackageHeaderExt,
        body: Exchange,
    ) -> State {
        trace!(
            "on_tunnel_event: head={}, head_ext={}, body={}",
            head,
            headext,
            body
        );

        debug_assert_eq!(headext.requestor(), &self.0.remote_device_id);

        let (err, state) = {
            if headext.to() == self.as_stack().local_device_id() {
                let state = self.0.connect_state.on_tunnel_event(head, headext, body);
                let err = match &state {
                    State::Connecting => ErrorCode::NEAR_ERROR_SUCCESS,
                    State::Established(_state) => ErrorCode::NEAR_ERROR_SUCCESS,
                    State::Dead => {
                        error!("failed to exchange by tunnel was dead.");
                        ErrorCode::NEAR_ERROR_TUNNEL_CLOSED
                    }
                };

                (err, Some(state))
            } else {
                error!("Request ID and target ID do not match, rejected, got:{}, expr:{}", self.as_stack().local_device_id(), headext.to(), );
                (ErrorCode::NEAR_ERROR_REFUSE, None)
            }
        };

        let arc_self = self.clone();
        async_std::task::block_on(async move {
            let from = &arc_self.0.remote_device_id;
            match err {
                ErrorCode::NEAR_ERROR_SUCCESS =>
                    info!("handshake package from: {{{}}} on tunnel: {{{}}}, tunnel state: Connecting => Activing", from, arc_self.0.interface),
                _ =>
                    error!("faiiled handshake package from: {{{}}} on tunnel: {{{}}}, with error: {}", from, arc_self.0.interface, err),
            }

            if let Ok((sequence, package)) = 
                self.as_stack()
                    .build_package(
                        BuildPackageV1 {
                            target: Some(arc_self.0.remote_device_id.clone()),
                            body: AnyNamedRequest::with_acktunnel(AckTunnel {
                                result: err.into_u16(),
                                send_time: now(),
                            }),
                            ..Default::default()
                        }
                    )
                    .await
                    .map_err(| err | {
                        error!(
                            "failed build ACKTunnel package to {} with {}",
                            arc_self.0.interface, err
                        );
                        err
                    }) {

                // PackageBuilder::build_head(seq.clone())
                //     .build_topic(
                //         None,
                //         stack.local_device_id().clone(),
                //         arc_self.0.remote_device_id.clone(),
                //         None,
                //     )
                //     .build_body(AnyNamedRequest::with_acktunnel(AckTunnel {
                //         result: err.into_u16(),
                //         send_time: now(),
                //     }))
                //     .build(None)
                //     .await
                //     .map_err(|err| {
                //         error!(
                //             "failed build ACKTunnel package to {} with {}",
                //             arc_self.0.interface, err
                //         );
                //         err
                //     }) {
                let _ = 
                    arc_self
                        .post_message((sequence, package))
                        .await
                        .map(|_| debug!("succeed send ACKTunnel pacage to {}", arc_self.0.interface))
                        .map_err(|err| error!("failed send data to {} with {}", arc_self.0.interface, err));

                if let Some(state) = state {
                    arc_self.sync_tunnel_state(state)
                } else {
                    State::Connecting
                }
            } else {
                State::Dead
            }
        })
    }
}

impl TunnelStateTrait<AckTunnel> for Tunnel {
    fn on_tunnel_event(
        &self,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        body: AckTunnel,
    ) -> State {
        trace!(
            "on_tunnel_event: head={}, head_ext={}, body={}",
            head,
            head_ext,
            body
        );

        debug_assert_eq!(head_ext.requestor(), &self.0.remote_device_id);
        let seq = head.sequence().clone();

        let (err, state) = {
            let state = self.0.connect_state.on_tunnel_event(head, head_ext, body);
            let err = match &state {
                State::Connecting => ErrorCode::NEAR_ERROR_SUCCESS,
                State::Established(_state) => ErrorCode::NEAR_ERROR_SUCCESS,
                State::Dead => {
                    error!("failed to exchange by tunnel was dead.");
                    ErrorCode::NEAR_ERROR_TUNNEL_CLOSED
                }
            };

            (err, state)
        };

        let arc_self = self.clone();

        async_std::task::block_on(async move {
            let from = &arc_self.0.remote_device_id;
            match err {
                ErrorCode::NEAR_ERROR_SUCCESS => {
                    info!("handshake package from: {{{}}} on tunnel: {{{}}}, tunnel state: Activing => Established", &arc_self.0.remote_device_id, arc_self.0.interface);
                }
                _ => {
                    info!("failed handshake package from: {{{}}}, refused, tunnel state: Activing => Deaded", from);
                }
            }

            if let Ok((sequence, package)) = 
                self.as_stack()
                    .build_package(
                        BuildPackageV1 {
                            target: Some(arc_self.0.remote_device_id.clone()),
                            body: AnyNamedRequest::with_ackacktunnel(AckAckTunnel {
                                sequence: seq,
                                result: err.into_u16(),
                                send_time: now(),
                            }),
                            ..Default::default()
                        }
                    )
                    .await
                    .map_err(| err | {
                        error!(
                            "failed build ACKTunnel package to {} with {}",
                            arc_self.0.interface, err
                        );
                        err
                    }) {

                // PackageBuilder::build_head(seq.clone())
                //     .build_topic(
                //         None,
                //         stack.local_device_id().clone(),
                //         arc_self.0.remote_device_id.clone(),
                //         None,
                //     )
                //     .build_body(AnyNamedRequest::with_ackacktunnel(AckAckTunnel {
                //         result: err.into_u16(),
                //         send_time: now(),
                //     }))
                //     .build(None)
                //     .await
                //     .map_err(|err| {
                //         error!(
                //             "failed build ACKACKTunnel package to {} with {}",
                //             arc_self.0.interface, err
                //         );
                //         err
                //     }) {
                let _ = 
                    arc_self
                        .post_message((sequence, package))
                        .await
                        .map(|_| debug!("succeed send ACKACKTunnel pacage to {}", arc_self.0.interface))
                        .map_err(|err| error!("failed send data to {} with {}", arc_self.0.interface, err));

                arc_self.sync_tunnel_state(state)
            } else {
                State::Dead
            }
        })
    }
}

impl TunnelStateTrait<AckAckTunnel> for Tunnel {
    fn on_tunnel_event(
        &self,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        body: AckAckTunnel,
    ) -> State {
        trace!(
            "on_tunnel_event: head={}, head_ext={}, body={}",
            head,
            head_ext,
            body
        );
        debug_assert_eq!(head_ext.requestor(), &self.0.remote_device_id);

        let state = self.0.connect_state.on_tunnel_event(head, head_ext, body);
        let _ = 
            match &state {
                State::Connecting => Ok(()),
                State::Established(_state) => {
                    info!("handshake package from: {{{}}} on tunnel: {{{}}}, tunnel state: Activing => Established", &self.0.remote_device_id, self.0.interface);
                    Ok(())
                }
                State::Dead => {
                    let error_message = format!("failed to exchange by [{}] was dead.", self);
                    error!("{}", error_message);
                    Err(NearError::new(
                        ErrorCode::NEAR_ERROR_TUNNEL_CLOSED,
                        error_message,
                    ))
                }
            };

        self.sync_tunnel_state(state)
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

    fn peer_id(&self) -> &DeviceId {
        &self.0.remote_device_id
    }

    fn ptr_eq(&self, other: &DynamicTunnel) -> bool {
        let tunnel: &Tunnel = other.as_ref();
        Arc::ptr_eq(&self.0, &tunnel.0)
    }

    fn is_closed(&self) -> bool {
        self.0.interface.is_closed()
    }

    fn reset(&self) {}

    fn update_time(&self) -> Timestamp {
        0
    }

    // async fn wait_active(&self) -> State {
    //     match self.0.connect_state.wait_active().await {
    //         State::Connecting => State::Connecting,
    //         State::Dead => State::Dead,
    //         State::Established(exchange_data) => {
    //             let guard = &epoch::pin();
    //             let curr = self.0.tunnel_state.load(std::sync::atomic::Ordering::SeqCst, guard);
    //             match unsafe { curr.as_ref().unwrap() } {
    //                 TunnelStateImpl::Connecting => {
    //                     let _ = self.0.tunnel_state.compare_exchange(curr,
    //                                                                  Owned::new(TunnelStateImpl::Actived(exchange_data)),
    //                                                                  std::sync::atomic::Ordering::SeqCst,
    //                                                                  std::sync::atomic::Ordering::SeqCst,
    //                                                                  guard);
    //                 }
    //                 _ => { /* ignore */ }
    //             }
    //             State::Established(())
    //         }
    //     }
    // }
}
