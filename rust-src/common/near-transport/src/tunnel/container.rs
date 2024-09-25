use log::{debug, error, info, trace, warn};
use std::{
    collections::{btree_map::Entry, BTreeMap, }, fmt::Write, sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    }, time::Duration
};

use near_base::{sequence::SequenceString, *};

use crate::{
    h::OnBuildPackage, network::{
        DataContext, DynamicInterface, Interface, TcpInterface, TcpPackageEventTrait, UdpInterface,
        UdpPackageEventTrait,
    }, 
    package::{
        Ack, AckAck, AckAckTunnel, AckTunnel, AnyNamedRequest, CreateVeriferTrait, Data, DynamicPackage, Exchange, MajorCommand, PackageDataSet, PackageHeader, PackageHeaderExt, StunReq
    }, 
    process::PackageEstablishedTrait, stack::BuildPackageV1, tunnel::{message::MessageResult, p::TunnelVerifier}, InterfaceMetaTrait, PackageEventTrait, Stack
};

use super::{message::Message, PostMessageTrait, TunnelManager};
use super::p::{OnRecvMessageCallback, OnSendMessageCallback};
use super::tunnel::{DynamicTunnel, State, TunnelStateTrait};
use super::{
    tcp::Tunnel as TcpTunnel, udp::Tunnel as UdpTunnel, TcpConfig, TunnelEventTrait, UdpConfig,
};

#[derive(Clone)]
pub struct Config {
    pub connect_timeout: Duration,
    pub recyle_timeout: Duration,
    pub resend_interval: Duration,
    pub resend_timeout: Duration,
    pub tcp: TcpConfig,
    pub udp: UdpConfig,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(30),
            recyle_timeout: Duration::from_millis(500),
            resend_interval: Duration::from_millis(120),
            resend_timeout: Duration::from_secs(5),
            tcp: Default::default(),
            udp: Default::default(),
        }
    }
}

enum TunnelContainerStateImpl {
    Connecting(StateWaiter),
    Actived,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct MessageTag {
    sequence: SequenceString,
    timestamp: Timestamp
}

impl MessageTag {
    pub fn new(
        sequence: SequenceString,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            sequence,
            timestamp,
        }
    }
}

type MessageRef = Arc<Message>;

#[derive(Clone)]
struct MessageSender {
    sender: DynamicTunnel,
    message: MessageRef,
}

struct TunnelMessages {
    tunnel: TunnelContainer,
    message_recv_center: RwLock<BTreeMap<MessageTag, MessageRef>>,
    message_send_center: RwLock<BTreeMap<MessageTag, MessageSender>>,
}

impl TunnelMessages {
    pub fn new(tunnel: TunnelContainer) -> Self {
        Self {
            tunnel,
            message_recv_center: RwLock::new(BTreeMap::new()),
            message_send_center: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn append(
        &self, 
        tunnel: DynamicTunnel, 
        sequence: SequenceString, 
        mut dataset: PackageDataSet
    ) {
        // exclude ack & ackack data context
        let dataset: Vec<DataContext> = 
            dataset
                .take_dataset()
                .into_iter()
                .map(|(context, _)| context)
                .filter(| context | {
                    match context.head.major_command() {
                        MajorCommand::Ack | MajorCommand::AckAck => false,
                        _ => true,
                    }
                })
                .collect();

        if dataset.is_empty() {
            return;
        }

        let timestamp = dataset.get(0).unwrap().head.timestamp();

        {
            let message = &mut *self.message_send_center.write().unwrap();

            let mut debug_info = String::default();
            dataset.iter()
                .for_each(| data | {
                    let _ = 
                        debug_info.write_fmt(
                            format_args!(
                                "data_context: requestor: {}, tunnel-target: {}, target: {}, index: {}", 
                                data.head_ext.requestor(), 
                                tunnel.peer_id(), 
                                data.head_ext.to(),
                                data.head.index())
                        );
                });

            match message.entry(MessageTag::new(sequence, timestamp)) {
                Entry::Occupied(exist) => {
                    let resend_timeout = self.tunnel.as_stack().config().tunnel.container.resend_timeout.as_micros() as u64;
                    let now = now();

                    if now - exist.get().message.message_created_time() > resend_timeout {
                        debug!("{} message timeout, so it will clean.", sequence);
                        let _ = exist.remove_entry();
                    } else {
                        debug!("updated message callback: sequence: {}, timestamp: {}, {debug_info}", sequence, timestamp);
                        let indexs: Vec<u8> = dataset.into_iter().map(|data| data.head.index()).collect();
                        exist.get().message.update_timestamp(&indexs, now);
                    }
                }
                Entry::Vacant(empty) => {
                    debug!("append message callback: sequence: {}, timestamp: {}, {debug_info}", sequence, timestamp);
                    let _ = 
                        empty.insert(MessageSender{
                            sender: tunnel,
                            message: MessageRef::new(Message::with_message(dataset)),
                        });
                }
            }
        }

        self.tunnel.0.manager.append_resender(self.tunnel.clone());
    }
}

#[async_trait::async_trait]
impl OnRecvMessageCallback for TunnelMessages {
    async fn on_callback(
        &self,
        tunnel: DynamicTunnel,
        mut data_context: DataContext,
    ) -> NearResult<()> {
        trace!(
            "on_recv_callback: tunnel={}, data_context={}",
            tunnel,
            data_context
        );

        if let None = data_context.head_ext.from.creator_remote {
            data_context.head_ext = 
                data_context
                    .head_ext
                    .set_endpoint(Some(tunnel.local().clone()), Some(tunnel.remote().clone()));
        }

        let head_ref = &data_context.head;
        let head_ext_ref = &data_context.head_ext;
        let sequence = head_ref.sequence().clone();

        self.tunnel.send_ack_package(tunnel.clone(), head_ref, head_ext_ref).await;

        // if head_ref.count() == 1 {
        //     self.tunnel.on_package(tunnel, data_context.try_into()?)
        // } else {
        {
            let message_ref = {
                match self
                        .message_recv_center
                        .write().unwrap()
                        .entry(MessageTag::new(head_ref.sequence().clone(), head_ref.timestamp()))
                {
                    Entry::Occupied(founed) => founed.get().clone(),
                    Entry::Vacant(empty) => {
                        let message = MessageRef::new(Message::new(data_context.head.count()));
                        empty.insert(message.clone());
                        message
                    }
                }
            };

            if let Ok(MessageResult::Finished(message_array)) =
                message_ref.push_context(data_context) {
                self.tunnel
                    .on_package(
                        tunnel, 
                        DataContext::merge(message_array)
                            .parse(self.tunnel.clone())
                            .await
                            .map_err(| err | {
                                let error_string = format!("failed parse package with err:{}", err);
                                error!("{error_string}, sequence: {}", sequence);
                                err
                            })?
                    )
                    .await
            } else {
                Ok(())
            }
        }
    }
}

#[async_trait::async_trait]
impl OnSendMessageCallback<(Ack, Timestamp)> for TunnelMessages {
    async fn on_callback(
        &self,
        tunnel: DynamicTunnel,
        data_context: (Ack, Timestamp),
    ) -> NearResult<()> {
        let (ack, timestamp) = data_context;
        trace!(
            "on_send_callback: tunnel={}, ack={}, timestamp={}",
            tunnel,
            ack,
            timestamp
        );

        let message_sender = 
            self.message_send_center
                .read().unwrap()
                .get(&MessageTag::new(ack.sequence, timestamp))
                .cloned()
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "invalid sequence")
                })?;

        if let Ok(_) = message_sender.message.push_index(ack.index) {
            Ok(())
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, "ignore"))
        }
    }
}

pub trait MessageEventTrait {
    fn on_succeed(&self, tunnel: DynamicTunnel, sequence: SequenceString, package: PackageDataSet);
    fn on_failure(&self, tunnel: DynamicTunnel, sequence: SequenceString, package: PackageDataSet, e: NearError);
}

struct TunnelContainerState {
    #[allow(unused)]
    last_update: Timestamp,
    container_state: TunnelContainerStateImpl,
    tunnel_entries: BTreeMap<EndpointPair, DynamicTunnel>,
}

#[derive(Default)]
struct RecyleState {
    recyled_interval: Timestamp,
    endpoints: Vec<EndpointPair>,
}

struct TunnelContainerImpl {
    manager: TunnelManager,
    remote: ObjectId,
    online: AtomicBool,
    #[allow(unused)]
    aes_key: AesKey,
    state: RwLock<TunnelContainerState>,
    recyle_state: RwLock<RecyleState>,
    tunnel_message: Option<TunnelMessages>,
}

#[derive(Clone)]
pub struct TunnelContainer(Arc<TunnelContainerImpl>);

impl TunnelContainer {
    pub(super) fn new(mgr: TunnelManager, remote: ObjectId) -> Self {
        let tunnel = Self(Arc::new(TunnelContainerImpl {
            manager: mgr,
            remote,
            online: AtomicBool::new(false),
            aes_key: AesKey::generate(),
            state: RwLock::new(TunnelContainerState {
                last_update: now(),
                container_state: TunnelContainerStateImpl::Connecting(StateWaiter::new()),
                tunnel_entries: BTreeMap::new(),
            }),
            recyle_state: RwLock::new(Default::default()),
            tunnel_message: None,
        }));

        let tunnel_message = TunnelMessages::new(tunnel.clone());

        let mut_tunnel = unsafe { &mut *(Arc::as_ptr(&tunnel.0) as *mut TunnelContainerImpl) };
        mut_tunnel.tunnel_message = Some(tunnel_message);

        tunnel
    }

    #[inline]
    #[allow(unused)]
    pub fn aes_key(&self) -> &AesKey {
        &self.0.aes_key
    }

    #[inline]
    pub(self) fn as_stack(&self) -> &Stack {
        &self.0.manager.as_stack()
    }

    #[inline]
    pub(crate) fn remote_id(&self) -> &ObjectId {
        &self.0.remote
    }

    #[inline]
    pub(self) fn message_center(&self) -> &TunnelMessages {
        self.0.tunnel_message.as_ref().unwrap()
    }

    pub(self) fn tunnel_of(&self, ep_pair: &EndpointPair) -> Option<DynamicTunnel> {
        self.0
            .state
            .read()
            .unwrap()
            .tunnel_entries
            .get(ep_pair)
            .map(|tunnel| tunnel.clone())
    }

    pub(self) fn create_tunnel(
        &self,
        ep_pair: &EndpointPair,
        remote: &ObjectId,
        interface: DynamicInterface,
    ) -> NearResult<DynamicTunnel> {
        let state = &mut *self.0.state.write().unwrap();
        match state.tunnel_entries.get(ep_pair) {
            Some(tunnel) => Ok(tunnel.clone()),
            None => {
                let tunnel = {
                    if ep_pair.is_tcp() {
                        let tunnel = TcpTunnel::new(
                            self.clone(),
                            self.as_stack().clone(),
                            ep_pair.clone(),
                            remote.clone(),
                            AsRef::<TcpInterface>::as_ref(&interface).clone(),
                        );
                        Ok(DynamicTunnel::new(tunnel))
                    } else if ep_pair.is_udp() {
                        let tunnel = UdpTunnel::new(
                            self.clone(),
                            self.as_stack().clone(),
                            ep_pair.clone(),
                            remote.clone(),
                            AsRef::<UdpInterface>::as_ref(&interface).clone(),
                        );
                        Ok(DynamicTunnel::new(tunnel))
                        // Err(NearError::new(
                        //     ErrorCode::NEAR_ERROR_UNKNOWN_PROTOCOL,
                        //     format!("{} unknown protocol.", remote.to_string()),
                        // ))
                    } else {
                        Err(NearError::new(
                            ErrorCode::NEAR_ERROR_UNKNOWN_PROTOCOL,
                            format!("{} unknown protocol.", remote.to_string()),
                        ))
                    }
                }?;

                state.tunnel_entries.insert(ep_pair.clone(), tunnel.clone());
                Ok(tunnel)
            }
        }
    }

    pub fn close_tunnel(&self, tunnel: DynamicTunnel) {
        let need_recyle = 
            if let Some(remove_tunnel) = 
                self.0.state.write().unwrap()
                    .tunnel_entries
                    .remove(&EndpointPair::new(tunnel.local_endpoint(), tunnel.remote_endpoint())) {

                if remove_tunnel.local().is_tcp() {
                    info!("{remove_tunnel} push into recyle_state.");
                    let mut_recyle = &mut *self.0.recyle_state.write().unwrap();
                    mut_recyle.recyled_interval = now();
                    mut_recyle.endpoints.push(EndpointPair::new(remove_tunnel.local_endpoint(), remove_tunnel.remote_endpoint()));
                    true
                } else {
                    info!("{remove_tunnel} will close and it don't need recyle.");
                    false
                }
            } else {
                false
            };

        if need_recyle {
            self.0.manager.append_recyle(self.clone());
        }
    }

    pub(self) async fn send_ack_package(
        &self,
        tunnel: DynamicTunnel, 
        head: &PackageHeader, 
        head_ext: &PackageHeaderExt
    ) {
        match head.major_command() {
            MajorCommand::Ack | MajorCommand::AckAck => return,
            _ => {},
        };

        let sequence = head.sequence().clone();
        let this = self;
        let remote = head_ext.requestor().clone();
        let index = head.index();

        if let Ok((_, package)) = 
            this.as_stack()
                .build_package(
                    BuildPackageV1 {
                        target: Some(remote),
                        timestamp: Some(head.timestamp()),
                        body: AnyNamedRequest::with_ack(Ack {
                                sequence,
                                index,
                                timestamp: now(),
                        }),
                        ..Default::default()
                    }
                )
                .await
                .map_err(| err | {
                    error!(
                        "failed build ACK package to {} with {}, sequence: {}",
                        tunnel, err, sequence
                    );
                    err
                })
        {
            if let Err(e) = tunnel.post_message(package).await {
                error!("failed post ACK message with err: {e}, sequence: {}, timestamp: {}", sequence, head.timestamp());
            } else {
                debug!("successed post ACK message with sequence: {}, timestamp: {}", sequence, head.timestamp());
            }
        }

    }

    pub(self) async fn on_package(
        &self,
        tunnel: DynamicTunnel,
        package: DynamicPackage,
    ) -> NearResult<()> {
        let context = self.as_stack().clone();
        trace!("on_package: tunnel={}, data_context={}", tunnel, package);

        // send_ack(tunnel.clone(), package.as_head(), package.as_headext());
        // self.send_ack_package(tunnel.clone(), package.as_head(), package.as_headext()).await;
    
        match package.as_head().major_command() {
            MajorCommand::Exchange => {
                let (head, head_ext, mut body, _) = package.split::<Exchange>();

                let from_device = {
                    std::mem::replace(&mut body.from_device, any::AnyNamedObject::None)
                };

                match {
                    if tunnel.local().is_tcp() {
                        tunnel.clone_as_tunnel::<TcpTunnel>().on_tunnel_event(head, head_ext, body)
                    } else if tunnel.local().is_udp() {
                        tunnel.clone_as_tunnel::<UdpTunnel>().on_tunnel_event(head, head_ext, body)
                    } else {
                        unreachable!()
                    }
                } {
                    State::Connecting | State::Established(_) => {
                        debug!("{} connecting...", tunnel);
                        match from_device {
                            any::AnyNamedObject::Device(o) | 
                            any::AnyNamedObject::Service(o) => {
                                self.as_stack().cacher_manager().add(&o);
                            }
                            _ => { /* ignore */ }
                        }
                    }
                    State::Dead => { debug!("{} dead", tunnel); },
                }
            }
            MajorCommand::AckTunnel => {
                let (head, head_ext, body, _) = package.split::<AckTunnel>();
                if tunnel.local().is_tcp() {
                    match tunnel.clone_as_tunnel::<TcpTunnel>().on_tunnel_event(head, head_ext, body)
                    {
                        State::Established(_) => {
                            debug!("{} established", tunnel);
                            self.as_stack().on_established(tunnel).await;
                        }
                        State::Connecting => { debug!("{} connecting", tunnel); },
                        _ => { debug!("{} dead", tunnel); },
                    }
                } else if tunnel.local().is_udp() {
                    match tunnel.clone_as_tunnel::<UdpTunnel>().on_tunnel_event(head, head_ext, body)
                    {
                        State::Established(_) => {
                            debug!("{} established", tunnel);
                            self.as_stack().on_established(tunnel).await;
                        }
                        State::Connecting => { debug!("{} connecting", tunnel); },
                        _ => { debug!("{} dead", tunnel); },
                    }
                } else {
                    unreachable!()
                }
            }
            MajorCommand::AckAckTunnel => {
                let (head, head_ext, body, _) = package.split::<AckAckTunnel>();
                if tunnel.local().is_tcp() {
                    match tunnel
                            .clone_as_tunnel::<TcpTunnel>()
                            .on_tunnel_event(head, head_ext, body)
                    {
                        State::Established(_) => {
                            debug!("{} established", tunnel);
                            self.as_stack().on_established(tunnel).await;
                        }
                        State::Connecting => debug!("{} connecting", tunnel),
                        _ => debug!("{} dead", tunnel),
                    }
                } else if tunnel.local().is_udp() {
                    match tunnel
                            .clone_as_tunnel::<UdpTunnel>()
                            .on_tunnel_event(head, head_ext, body)
                    {
                        State::Established(_) => {
                            debug!("{} established", tunnel);
                            self.as_stack().on_established(tunnel).await;
                        }
                        State::Connecting => debug!("{} connecting", tunnel),
                        _ => debug!("{} dead", tunnel),
                    }
                } else {
                    unreachable!()
                }
            }
            MajorCommand::Ack => {
                let this = self.clone();
                let (head, head_ext, ack, _) = package.split::<Ack>();

                debug!(
                    "success recv ACK package from {}, timestamp:{}, head_ext: {}, data: {}",
                    tunnel,
                    head.timestamp(),
                    head_ext,
                    ack
                );

                let index = ack.index;

                if let Ok(_) = 
                    OnSendMessageCallback::on_callback(
                        &this,
                        tunnel.clone(),
                        (ack, head.timestamp()),
                    )
                    .await {
                        let timestamp = head.timestamp();
                        let (_, sequence) = head.split();
                        if let Ok((_, package)) = 
                            this.as_stack()
                                .build_package(
                                    BuildPackageV1 {
                                        target: Some(this.0.remote.clone()),
                                        timestamp: Some(timestamp),
                                        body: AnyNamedRequest::with_ackack(AckAck {
                                            sequence: sequence,
                                            index,
                                            errno: ErrorCode::NEAR_ERROR_SUCCESS.into_u16(),
                                        }),
                                        ..Default::default()
                                    }
                                )
                                .await
                                .map_err(| err | {
                                    error!("failed build ACKACK package to {} with {}", tunnel, err);
                                    err
                                }) {
                            if let Err(e) = tunnel.post_message(package).await {
                                error!("failed post ACKACK message with err: {e}");
                            }
                        }
                    }
            }
            MajorCommand::AckAck => {
                let (head, _, ackack, _) = package.split::<AckAck>();

                debug!(
                    "success recv ACKACK package from {}, sequence: {}, data: {}",
                    tunnel,
                    head.sequence(),
                    ackack
                );

                // self.message_center().message_recv_center.write().unwrap()
            }
            MajorCommand::Stun => {
                async_std::task::spawn(async move {
                    let (head, head_ext, body, signature) = package.split::<StunReq>();

                    if let Some(signature) = signature {
                        let _ = 
                            context
                                .on_package_event(
                                    tunnel.clone(), 
                                    head, 
                                    head_ext, 
                                    (body, signature)
                                )
                                .await;
                    } else {
                        warn!("not found signaure data.")
                    }
                });
            }
            // MajorCommand::Ping => {
            //     async_std::task::spawn(async move {
            //         let (head, head_ext, body, signature) = package.split::<Ping>();

            //         if let Some(signature) = signature {
            //             let _ = 
            //                 context
            //                     .on_package_event(
            //                         tunnel.clone(), 
            //                         head, 
            //                         head_ext, 
            //                         (body, signature)
            //                     )
            //                     .await;
            //         } else {
            //             warn!("not found signaure data.")
            //         }
            //     });
            // }
            // MajorCommand::PingResp => {
            //     async_std::task::spawn(async move {
            //         let (head, head_ext, body, signature) = package.split::<PingResp>();

            //         if let Some(signature) = signature {
            //             let _ = context
            //                 .on_package_event(tunnel.clone(), head, head_ext, (body, signature))
            //                 .await;
            //         } else {
            //             warn!("not found signaure data.")
            //         }
            //     });
            // }
            // MajorCommand::CallCommand => {
            //     let minor_command = 
            //         CallSubCommand::from_str(
            //             package.as_headext().topic().map(| topic | topic.as_str()).unwrap_or("")
            //         )?;

            //     match minor_command {
            //         CallSubCommand::Call => {
            //             let (head, head_ext, call_req, signature) = package.split::<CallReq>();

            //             if let Some(signature) = signature {
            //                 let _ = context
            //                     .on_package_event(tunnel.clone(), head, head_ext, (call_req, signature))
            //                     .await;
            //             } else {
            //                 warn!("not found signaure data.")
            //             }
            //         }
            //         CallSubCommand::CallResp => {
            //             let (head, head_ext, call_resp, signature) = package.split::<CallResp>();

            //             if let Some(signature) = signature {
            //                 let _ = context
            //                     .on_package_event(tunnel.clone(), head, head_ext, (call_resp, signature))
            //                     .await;
            //             } else {
            //                 warn!("not found signaure data.")
            //             }
            //         }
            //         CallSubCommand::Called => {
            //             let (head, head_ext, called_req, signature) = package.split::<CalledReq>();

            //             if let Some(signature) = signature {
            //                 let _ = context
            //                     .on_package_event(tunnel.clone(), head, head_ext, (called_req, signature))
            //                     .await;
            //             } else {
            //                 warn!("not found signaure data.")
            //             }
            //         }
            //         CallSubCommand::CalledResp => {
            //             let (head, head_ext, called_resp, signature) = package.split::<CalledResp>();

            //             if let Some(signature) = signature {
            //                 let _ = context
            //                     .on_package_event(tunnel.clone(), head, head_ext, (called_resp, signature))
            //                     .await;
            //             } else {
            //                 warn!("not found signaure data.")
            //             }
            //         }
            //     }
            // }
            // MajorCommand::Call => {
            //     async_std::task::spawn(async move {
            //         let (head, head_ext, call_req, signature) = package.split::<CallReq>();

            //         if let Some(signature) = signature {
            //             let _ = context
            //                 .on_package_event(tunnel.clone(), head, head_ext, (call_req, signature))
            //                 .await;
            //         } else {
            //             warn!("not found signaure data.")
            //         }
            //     });
            // }
            // MajorCommand::CallResp => {
            //     async_std::task::spawn(async move {
            //         let (head, head_ext, call_resp, signature) = package.split::<CallResp>();

            //         if let Some(signature) = signature {
            //             let _ = context
            //                 .on_package_event(tunnel.clone(), head, head_ext, (call_resp, signature))
            //                 .await;
            //         } else {
            //             warn!("not found signaure data.")
            //         }
            //     });
            // }
            MajorCommand::Request | MajorCommand::Response => {
                let (head, head_ext, data, _sign_data) = package.split::<Data>();

                {
                    async_std::task::spawn(async move {
                        let stack = context;

                        // todo: need verify signdata
                        let v: Vec<u8> = data.into();

                        if let Err(e) = 
                            stack
                                .on_package_event(tunnel.clone(), head, head_ext, v)
                                .await
                        {
                            warn!(
                                "failed on_package_event with err = {} from tunnel = {}",
                                e, tunnel
                            );
                        }
                    });
                }
            }
            MajorCommand::None => unreachable!(),
        }

        Ok(())
    }

    pub(super) fn sync_tunnel_state(&self, tunnel: DynamicTunnel, state: &State) {
        match state {
            State::Connecting => { /* Ignore */ }
            State::Dead => {
                // TODO
                // remote tunnel from entries
                let state = &mut *self.0.state.write().unwrap();

                match state.tunnel_entries.entry(EndpointPair::new(
                    tunnel.local().clone(),
                    tunnel.remote().clone(),
                )) {
                    Entry::Vacant(_not_found) => {
                        // ignore
                    }
                    Entry::Occupied(founded) => {
                        founded.remove();
                    }
                }
            }
            State::Established(_) => {
                self.active();
            }
        }
    }

    pub(self) fn is_online(&self) -> bool {
        self.0.online.load(Ordering::SeqCst)
    }

    pub(crate) fn to_state(&self) -> State {
        let state = &*self.0.state.read().unwrap();
        match &state.container_state {
            TunnelContainerStateImpl::Connecting(_) => State::Connecting,
            TunnelContainerStateImpl::Actived => State::Established(()),
        }
    }

    pub(crate) async fn wait_active(&self) -> State {
        if self.0.online.load(Ordering::SeqCst) {
            return self.to_state();
        }

        let (state, waiter) = {
            let state = &mut *self.0.state.write().unwrap();
            match &mut state.container_state {
                TunnelContainerStateImpl::Connecting(waiter) => {
                    (State::Connecting, Some(waiter.new_waiter()))
                }
                TunnelContainerStateImpl::Actived => (State::Established(()), None),
            }
        };

        if let Some(waiter) = waiter {
            StateWaiter::wait(waiter, || self.to_state()).await
        } else {
            state
        }
    }

    fn active(&self) {
        if let Ok(_) =
            self.0
                .online
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        {
            let waker = {
                let state = &mut *self.0.state.write().unwrap();
                let container_state = &mut state.container_state;

                match container_state {
                    TunnelContainerStateImpl::Connecting(waiter) => {
                        let waker = Some(waiter.transfer());
                        *container_state = TunnelContainerStateImpl::Actived;
                        waker
                    }
                    TunnelContainerStateImpl::Actived => None,
                }
            };

            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }

    fn dead(&self) {
        if let Ok(_) =
            self.0
                .online
                .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        {
            let waker = {
                let state = &mut *self.0.state.write().unwrap();
                let container_state = &mut state.container_state;

                match container_state {
                    TunnelContainerStateImpl::Connecting(waiter) => {
                        let waker = Some(waiter.transfer());
                        *container_state = TunnelContainerStateImpl::Actived;
                        waker
                    }
                    TunnelContainerStateImpl::Actived => {
                        *container_state = TunnelContainerStateImpl::Connecting(StateWaiter::new());
                        None
                    }
                }
            };

            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }
}

impl TunnelContainer {
    // pub fn on_package_event(&self, package: DynamicPackage)
}

#[async_trait::async_trait]
impl TcpPackageEventTrait for TunnelContainer {
    fn on_connected(&self, interface: TcpInterface, remote: &DeviceObject) {
        debug_assert!(remote.object_id() == &self.0.remote);

        let ep_pair = interface.endpoint_pair();
        if let Ok(tunnel) = self
            .create_tunnel(
                &ep_pair,
                remote.object_id(),
                DynamicInterface::new(interface),
            )
            .map(|tunnel| tunnel.clone())
            .map_err(|err| error!("failed create tunnel with err: {} in on_connected.", err))
        {
            if tunnel.local().is_tcp() {
                tunnel.clone_as_tunnel::<TcpTunnel>().active(remote);
            } else {
                unreachable!("don't reach here.")
            }
        }
    }

    fn on_closed(&self, interface: &TcpInterface, remote: &ObjectId) {
        trace!("on_closed, remote:{remote}, interface: {interface}");

        let ep_pair = interface.endpoint_pair();
        let v = match self.0.state.write().unwrap().tunnel_entries.entry(ep_pair) {
            Entry::Occupied(exist) => {
                info!("{remote} push into recyle_state.");
                let (k, _v) = exist.remove_entry();
                let mut_recyle = &mut *self.0.recyle_state.write().unwrap();
                mut_recyle.recyled_interval = now();
                mut_recyle.endpoints.push(k);
                Some(_v)
            }
            _ => {
                warn!("not found tunnel with interface: {interface}.");
                None
            }
        };

        if let Some(_) = v {
            self.0.manager.append_recyle(self.clone());
        }
    }

    async fn on_tcp_package(&self, interface: TcpInterface, data_context: DataContext) -> NearResult<()> {
        trace!(
            "on_tcp_package: interface={}, data_context={}",
            interface,
            data_context
        );

        let ep_pair = interface.endpoint_pair();
        let tunnel = {
            match self.tunnel_of(&ep_pair) {
                Some(tunnel) => Ok(tunnel),
                None => {
                    self.create_tunnel(&ep_pair, &self.0.remote, DynamicInterface::new(interface))
                }
            }
        }?;

        OnRecvMessageCallback::on_callback(self, tunnel, data_context).await
    }
}

#[async_trait::async_trait]
impl UdpPackageEventTrait<Endpoint> for TunnelContainer {
    fn on_connected(
        &self,
        interface: UdpInterface,
        remote: &DeviceObject,
        remote_endpoint: Endpoint,
    ) {
        debug_assert!(remote.object_id() == &self.0.remote);

        let ep_pair = EndpointPair::new(interface.local().clone(), remote_endpoint);
        if let Ok(tunnel) = 
            self
                .create_tunnel(
                    &ep_pair,
                    remote.object_id(),
                    DynamicInterface::new(interface),
                )
                .map(|tunnel| tunnel.clone())
                .map_err(|err| error!("failed create tunnel with err: {} in on_connected.", err))
        {
            if tunnel.local().is_udp() {
                tunnel.clone_as_tunnel::<UdpTunnel>().active(remote);
            } else {
                unreachable!("don't reach here.")
            }
        }
    }

    async fn on_udp_package(
        &self,
        interface: UdpInterface,
        data_context: DataContext,
        remote: Endpoint,
    ) -> NearResult<()> {
        trace!(
            "on_udp_package: interface={}, data_context={}",
            interface,
            data_context
        );

        let ep_pair = EndpointPair::new(interface.local().clone(), remote);
        let tunnel = {
            match self.tunnel_of(&ep_pair) {
                Some(tunnel) => Ok(tunnel),
                None => {
                    self.create_tunnel(&ep_pair, &self.0.remote, DynamicInterface::new(interface))
                }
            }
        }?;

        OnRecvMessageCallback::on_callback(self, tunnel, data_context).await
    }
}

impl TunnelContainer {

    pub(self) async fn wait_and_take_tunnel(&self) -> NearResult<DynamicTunnel> {
        if !self.is_online() {
            match async_std::future::timeout(
                        self.as_stack().config().tunnel.container.tcp.send_timeout,
                        self.wait_active(),
                    )
                    .await {
                Ok(r) => match r {
                    State::Established(_) => Ok(()),
                    State::Connecting => {
                        unreachable!()
                    }
                    State::Dead => {
                        let error_string = format!("Failed to actived remote: {}.", self.0.remote);
                        error!("{}.", error_string);
                        Err(NearError::new(
                            ErrorCode::NEAR_ERROR_UNACTIVED,
                            error_string,
                        ))
                    }
                },
                Err(_) => {
                    let error_string = format!("Timeout to actived remote: {}.", self.0.remote);
                    error!("{}", error_string);
                    Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
                }
            }
        } else {
            Ok(())
        }?;

        self.tunnel_random().ok_or_else(|| {
                self.dead();
                NearError::new(
                    ErrorCode::NEAR_ERROR_NO_AVAILABLE,
                    format!("{} tunnel no available.", self.0.remote),
                )
            })

    }

    pub(self) fn tunnel_random(&self) -> Option<DynamicTunnel> {
        let tunnel_array: Vec<DynamicTunnel> = {
            self.0
                .state
                .read().unwrap()
                .tunnel_entries
                .values()
                .map(| v | v.clone())
                .collect()
        };

        let tunnel_count = tunnel_array.len();
        if tunnel_count == 0 {
            warn!("Tunnel is not alive, please wait...");
            return None;
        }

        let mut recyle_entries = vec![];
        let tunnel_array_ref = tunnel_array.as_slice();
        let mut index = now() as usize % tunnel_count;
        let orig_index = index;

        let tunnel: Option<DynamicTunnel> = loop {
            let tunnel = &tunnel_array_ref[index];
            if !tunnel.is_closed() {
                break (Some(tunnel.clone()));
            } else {
                recyle_entries.push(EndpointPair::new(
                    tunnel.local().clone(),
                    tunnel.remote().clone(),
                ));

                index += 1;

                if index == tunnel_count {
                    index = 0;
                }
                if index == orig_index {
                    break (None);
                }
            }
        };

        if recyle_entries.len() > 0 {
            {
                let state = &mut *self.0.state.write().unwrap();

                for ep in recyle_entries.iter() {
                    let _ = state.tunnel_entries.remove(ep);
                }
            }

            {
                let mut_recyle = &mut *self.0.recyle_state.write().unwrap();

                mut_recyle.recyled_interval = now();
                mut_recyle.endpoints.append(&mut recyle_entries);
            }

            self.0.manager.append_recyle(self.clone());
        }

        tunnel
    }

}

// #[async_trait::async_trait]
// impl PostMessageTrait<(HeaderMeta, Data)> for TunnelContainer {
//     async fn post_message(&self, context: (HeaderMeta, Data)) -> NearResult<()> {
//         let (header_meta, body) = context;

//         let tunnel = self.wait_and_take_tunnel().await?;

//         debug!(
//             "post message to {tunnel}, sequence = {}",
//             header_meta.sequence(),
//         );
    
//         if tunnel.local().is_tcp() {
//             tunnel
//                 .clone_as_tunnel::<TcpTunnel>()
//                 .post_message((header_meta, body))
//                 .await
//         } else if tunnel.local().is_udp() {
//             tunnel
//                 .clone_as_tunnel::<UdpTunnel>()
//                 .post_message((header_meta, body))
//                 .await
//         } else {
//             unreachable!()
//         }
//     }
// }

#[async_trait::async_trait]
impl PostMessageTrait<(SequenceString, PackageDataSet)> for TunnelContainer {

    type R = ();

    async fn post_message(
        &self, 
        context: (SequenceString, PackageDataSet)
    ) -> NearResult<Self::R> {
        let (sequence, package) = context;

        let tunnel = 
            self.wait_and_take_tunnel()
                .await
                .map(| tunnel | {
                    debug!(
                        "post message to {tunnel}, sequence = {sequence}",
                    );
                    tunnel
                })?;

        if tunnel.local().is_tcp() {
            tunnel
                .clone_as_tunnel::<TcpTunnel>()
                .post_message((sequence, package))
                .await
        } else if tunnel.local().is_udp() {
            tunnel
                .clone_as_tunnel::<UdpTunnel>()
                .post_message((sequence, package))
                .await
        } else {
            unreachable!()
        }
            // .post_message(package)
            // .await

    }
}

impl MessageEventTrait for TunnelContainer {
    fn on_failure(&self, _tunnel: DynamicTunnel, _sequence: SequenceString, _dataset: PackageDataSet, _e: NearError) {
        error!("failed message event sequence: {_sequence}, with err: {_e}")
    }

    fn on_succeed(&self, tunnel: DynamicTunnel, sequence: SequenceString, dataset: PackageDataSet) {
        info!("successfully post message target: {}, sequence: {}", tunnel.peer_id(), sequence);
        self.message_center().append(tunnel, sequence, dataset);
    }
}

impl TunnelContainer {
    pub(crate) fn on_time_escape_for_recyle(&self, now: Timestamp) {
        let recyle_entries = {
            let recyle_timeout = self.as_stack().config().tunnel.container.recyle_timeout.as_micros() as u64;
            let mut_recyle = &mut *self.0.recyle_state.write().unwrap();

            if now - mut_recyle.recyled_interval > recyle_timeout {
                mut_recyle.recyled_interval = now;
                std::mem::replace(&mut mut_recyle.endpoints, vec![])
            } else {
                vec![]
            }
        };

        if recyle_entries.len() <= 0 {
            return;
        }

        {
            if recyle_entries.len() > 0 {
                debug!("recyle_entries-size: {}", recyle_entries.len());
                let arc_self = self.clone();
                async_std::task::spawn(async move {
                    let stack = arc_self.as_stack();
                    let mut futures_list = vec![];

                    for ep in recyle_entries.iter() {
                        futures_list
                            .push(stack.on_reconnect(ep.remote().clone(), &arc_self.0.remote));
                    }

                    let r = futures::future::join_all(futures_list).await;

                    let mut remain_entries = vec![];

                    for (idx, ep) in recyle_entries.iter().enumerate() {
                        if let Err(err) = r.get(idx).unwrap().as_ref() {
                            match err.errno() {
                                ErrorCode::NEAR_ERROR_NOTFOUND | ErrorCode::NEAR_ERROR_IGNORE => {},
                                _ => {
                                    remain_entries.push(ep.clone());
                                }
                            }
                        }
                    }

                    if remain_entries.len() > 0 {
                        let mut_recyle = &mut *arc_self.0.recyle_state.write().unwrap();

                        mut_recyle.recyled_interval = now;
                        mut_recyle.endpoints.append(&mut remain_entries);
                    } else {
                        arc_self.0.manager.remove_recyle(arc_self.remote_id());
                    }
                });
            }
        }
    }

    pub(crate) fn on_time_escape_for_resend(&self, now: Timestamp) {
        let (need_recyle_messages, all_message_senders) = {
            let mut need_sender_messages = vec![];
            let mut need_recyle_messages = vec![];
            let resend_timeout = self.as_stack().config().tunnel.container.resend_timeout.as_micros() as u64;
            let mut_message = &mut *self.message_center().message_send_center.write().unwrap();

            mut_message.retain(| _key, message | {
                if now - message.message.message_created_time() > resend_timeout {
                    // debug!("{} message timeout, so it will clean.", key.sequence);
                    need_recyle_messages.push(message.clone());
                    false
                } else {
                    need_sender_messages.push(message.clone());
                    true
                }
            });

            (need_recyle_messages, need_sender_messages)
        };

        let resend_messages_proc = | all_message_senders: Vec<MessageSender> | {
            if all_message_senders.len() == 0 {
                self.0.manager.remove_resender(self.remote_id());
                return;
            }

            // unfinished data
            let need_sender_messages: Vec<(DynamicTunnel, Vec<Arc<Option<DataContext>>>)> = {
                let resend_interval = self.as_stack().config().tunnel.container.resend_interval.as_micros() as u64;
                all_message_senders.iter()
                    .map(| message | {
                        let resend_message: Vec<Arc<Option<DataContext>>> = 
                            message.message.unfinished_context()
                                .into_iter()
                                .filter(| context | {
                                    if now > context.message_timestamp {
                                        now - context.message_timestamp >= resend_interval
                                    } else {
                                        false
                                    }
                                })
                                .map(| context | {
                                    let head_ref = &context.message.as_ref().as_ref().unwrap().head;
                                    debug!(
                                        "need resend: sequence: {}, command: {}, now: {}, timestmp: {}", 
                                        head_ref.sequence(), head_ref.major_command(),
                                        now,
                                        context.message_timestamp
                                    );
                                    context.message
                                })
                                .collect();
                        (message.sender.clone(), resend_message)
                    })
                    .collect()

            };

            if need_sender_messages.len() <= 0 {
                return;
            }

            async_std::task::spawn(async move {
                for (tunnel, dataset) in need_sender_messages {
                    let dataset: Vec<DataContext> = 
                        dataset
                            .iter()
                            .filter(|data| data.is_some())
                            .map(|data| data.as_ref().as_ref().unwrap().clone())
                            .collect();

                    if dataset.len() <= 0 {
                        continue;
                    }

                    let sequence = dataset.get(0).unwrap().head.sequence().clone();

                    if tunnel.local().is_tcp() {
                        let _ = 
                            tunnel
                                .clone_as_tunnel::<TcpTunnel>()
                                .post_message((sequence, dataset))
                                .await;
                    } else if tunnel.local().is_udp() {
                        let _ = 
                            tunnel
                                .clone_as_tunnel::<UdpTunnel>()
                                .post_message((sequence, dataset))
                                .await;
                    } else {
                        unreachable!()
                    }

                }
            });
        };

        let recyle_messages_proc = | need_recyle_messages: Vec<MessageSender> | {
            let this = self.clone();
            async_std::task::spawn(async move {
                let need_recyle_messages: Vec<(DynamicTunnel, Vec<DataContext>)> = 
                unsafe {
                    need_recyle_messages.into_iter()
                        .map(| message | {
                            let message_array: Vec<DataContext> = 
                                    message.message
                                        .unfinished_context()
                                        .into_iter()
                                        .filter(| context | context.message.is_some())
                                        .map(| context | {
                                                std::mem::replace(
                                                    &mut *(Arc::as_ptr(&context.message) as *mut Option<DataContext>), 
                                                    None
                                                )
                                                .unwrap()
                                        })
                                        .collect();
                            (message.sender, message_array)
                        })
                        .collect()
                };

                for (tunnel, dataes) in need_recyle_messages {
                    if dataes.len() > 0 {
                        let sequence = dataes.get(0).unwrap().head.sequence().clone();

                        // The tunnel maybe closed
                        this.close_tunnel(tunnel);

                        match PackageDataSet::try_from(dataes) {
                            Ok(package) => {
                                let _ = 
                                this.post_message((sequence.clone(), package))
                                    .await
                                    .map(| _ | {
                                        info!("successfully resender sequence: {sequence}");
                                    })
                                    .map_err(| err | {
                                        error!("faileure resender sequence: {sequence}");
                                        err
                                    });
                            }
                            Err(err) => {
                                error!("failed convert data to PackageDataSet with err: {err}, sequence: {sequence}")
                                // ignore
                            }
                        }
                    }
                }
                // for message in need_recyle_messages {
                //     let (tunnel, message) = message.split();

                //     let unfinished_context: Vec<DataContext> = 
                //         message.unfinished_context()
                //             .into_iter()
                //             .filter(| context | context.message.is_some())
                //             .map(| context | context.message.unwrap())
                //             .collect();

                //     if unfinished_context.len() > 0 {
                //         // The tunnel maybe closed
                //         this.close_tunnel(tunnel);
                //         // change tunnel to post
                //         // this.post_message(context)
                //         // this.as_stack().on_package_event(tunnel, head, head_ext, data)
                //     }
                // }
            });
        };

        resend_messages_proc(all_message_senders);
        recyle_messages_proc(need_recyle_messages);


        // {
        //     fn do_unfinished_context(tunnel_array: Vec<DynamicTunnel> , messages: Vec<Vec<Arc<Option<DataContext>>>>) {
        //         let index = 0usize;
        //         let tunnel_cnt = tunnel_array.len();
        //         if tunnel_cnt <= 0 {
        //             return;
        //         }

        //         async_std::task::spawn(async move {
        //             for dataset in messages {
        //                 let dataset: Vec<DataContext> = 
        //                     dataset
        //                         .iter()
        //                         .filter(|data| data.is_some())
        //                         .map(|data| data.as_ref().as_ref().unwrap().clone())
        //                         .collect();

        //                 if dataset.len() <= 0 {
        //                     continue;
        //                 }

        //                 let sequence = dataset.get(0).unwrap().head.sequence().clone();

        //                 let tunnel = tunnel_array.get(index % tunnel_cnt).unwrap();

        //                 if tunnel.local().is_tcp() {
        //                     let _ = 
        //                         tunnel
        //                             .clone_as_tunnel::<TcpTunnel>()
        //                             .post_message((sequence, dataset))
        //                             .await;
        //                 } else if tunnel.local().is_udp() {
        //                     let _ = 
        //                         tunnel
        //                             .clone_as_tunnel::<UdpTunnel>()
        //                             .post_message((sequence, dataset))
        //                             .await;
        //                 } else {
        //                     unreachable!()
        //                 }
        //             }
        //         });
   
        //     }

        //     do_unfinished_context(tunnel_array, unfinished_messages);
        // }

        // timeout
        // {
        //     let resend_timeout = 
        //         self.as_stack()
        //             .config()
        //             .tunnel
        //             .container
        //             .resend_timeout
        //             .as_micros() as u64;

        //     let timeout_messages: Vec<Vec<Arc<Option<DataContext>>>> = 
        //         all_messages.iter()
        //             .map(| message | {
        //                 message.unfinished_context()
        //                     .into_iter()
        //                     .filter(| context | {
        //                         if now > context.message_created {
        //                             now - context.message_created <= resend_timeout
        //                         } else {
        //                             false
        //                         }
        //                     })
        //                     .map(| context | {
        //                         debug!(
        //                             "need resend: sequence: {}, now: {}, timestmp: {}", 
        //                             context.message.as_ref().as_ref().unwrap().head.sequence(),
        //                             now,
        //                             context.message_timestamp
        //                         );
        //                         context.message
        //                     })
        //                     .collect()
        //             })
        //             .collect();
        // }

        ////////////////////////////////////////////////
        // let unfinished_context: Vec<(SequenceString, Vec<Arc<Option<DataContext>>>)> = {
        //     self.message_center()
        //         .message_send_center
        //         .read().unwrap()
        //         .iter()
        //         .map(|(message_tag, message)| {
        //             (
        //                 message_tag.sequence.clone(),
        //                 {
        //                     message.unfinished_context()
        //                         .into_iter()
        //                         .filter(| context | {
        //                             if now > context.message_timestamp {
        //                                 now - context.message_timestamp > resend_interval
        //                             } else {
        //                                 false
        //                             }
        //                         })
        //                         .map(| context | {
        //                             debug!(
        //                                 "need resend: sequence: {}, now: {}, timestmp: {}", 
        //                                 context.message.as_ref().as_ref().unwrap().head.sequence(),
        //                                 now,
        //                                 context.message_timestamp
        //                             );
        //                             context.message
        //                         })
        //                         .collect()
        //                 }
        //             )
        //         })
        //         .collect()
        // };

        // if unfinished_context.len() <= 0 {
        //     return;
        // }

        // let index = now as usize;
        // let count = tunnel_array.len();

    }
}

#[async_trait::async_trait]
impl OnRecvMessageCallback for TunnelContainer {
    async fn on_callback(
        &self,
        tunnel: DynamicTunnel,
        data_context: DataContext,
    ) -> NearResult<()> {
        OnRecvMessageCallback::on_callback(self.message_center(), tunnel, data_context).await
    }
}

#[async_trait::async_trait]
impl OnSendMessageCallback<(Ack, Timestamp)> for TunnelContainer {
    async fn on_callback(
        &self,
        tunnel: DynamicTunnel,
        data_context: (Ack, Timestamp),
    ) -> NearResult<()> {
        OnSendMessageCallback::on_callback(self.message_center(), tunnel, data_context).await
    }
}

#[async_trait::async_trait]
impl CreateVeriferTrait for TunnelContainer {
    async fn create_verifer_obj(&self, requestor: &ObjectId) -> NearResult<Box<dyn VerifierTrait>> {
        Ok(Box::new(TunnelVerifier::new(self.as_stack(), requestor).await?))
    }
}

#[derive(Clone)]
pub struct TunnelGuard(Arc<TunnelContainer>);

impl TunnelGuard {
    pub(super) fn new(tunnel: TunnelContainer) -> Self {
        Self(Arc::new(tunnel))
    }
}

impl std::ops::Deref for TunnelGuard {
    type Target = TunnelContainer;
    fn deref(&self) -> &TunnelContainer {
        &(*self.0)
    }
}
