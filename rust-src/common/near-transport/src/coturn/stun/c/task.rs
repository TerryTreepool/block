
use std::{fmt::Write, sync::{atomic::{AtomicU64, AtomicU8}, Arc, Mutex}, time::Duration};
use async_std::sync::RwLock;

use log::{debug, error, info, trace, warn};
use near_base::{now, sequence::SequenceString, DeviceObject, DeviceObjectSubCode, Endpoint, EndpointPair, ErrorCode, NearError, NearResult, ObjectId, ObjectTypeCode, ServiceObjectSubCode, StateWaiter};

use crate::{
        network::Udp, 
        package::{AnyNamedRequest, PackageDataSet, StunReq, StunType }, 
        process::PackageEstablishedTrait, 
        coturn::stun::p::CallTemplate, 
        tunnel::{DynamicTunnel, PostMessageTrait}, 
        InterfaceMetaTrait, 
        RequestorMeta, 
        Stack
    };

use super::{ping::PingManager, NetworkAccessType};

enum SessionInnerStatus {
    Connecting,
    Established(SessionEstablished),
    Dead,
}

impl std::fmt::Display for SessionInnerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", 
            match self {
                Self::Connecting => "Connecting",
                Self::Established(_) => "Established",
                Self::Dead => "Dead",
            }
        )
    }
}

struct SessionEstablished {
    remote_external: Option<Endpoint>,
    tunnel: DynamicTunnel,
}

// impl std::fmt::Display for SessionInnerStatus {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Connecting(c) => write!(f, "Connecting: remote host: {}", c.remote),
//             Self::Established(c) => write!(f, "Established: remote host: {}, remote mapping host: {:?}", c.remote, c.remote_external),
//             Self::Dead(c) => write!(f, "Dead : remote host: {}", c.remote),
//         }
//     }
// }

enum SessionNetwork {
    Tcp,
    Udp(Udp),
}

enum SessionRemoteObject {
    Service(DeviceObject),
    Device(DeviceObject),
}

impl TryFrom<DeviceObject> for SessionRemoteObject {
    type Error = NearError;

    fn try_from(value: DeviceObject) -> NearResult<Self> {
        match value.object_id().object_type_code()? {
            ObjectTypeCode::Device(o) if o == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => 
                Ok(Self::Device(value)),
            ObjectTypeCode::Service(o) if o == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_COTURN_MINER as u8 => 
                Ok(Self::Service(value)),
            _ => 
                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "invalid device object"))
        }
    }
}

impl SessionRemoteObject {
    pub fn as_remote(&self) -> &DeviceObject {
        match self {
            Self::Service(v) => v,
            Self::Device(v) => v,
        }
    }

    pub fn is_service(&self) -> bool {
        match self {
            Self::Service(_) => true,
            Self::Device(_) => false,
        }
    }
}

struct SessionStatus {

    remote_endpoint: Endpoint,
    remote_status: RwLock<SessionInnerStatus>, // 对端(SN)地址

    last_connect_time: AtomicU64,
    last_ping_time: AtomicU64,
    last_resp_time: AtomicU64,
}

impl SessionStatus {
    pub fn new(remote_endpoint: Endpoint) -> Self {
        Self {
            remote_endpoint,
            remote_status: RwLock::new(SessionInnerStatus::Connecting), // 对端(SN)地址
            last_connect_time: AtomicU64::new(0),
            last_ping_time: AtomicU64::new(now()),
            last_resp_time: AtomicU64::new(0),
        }
    }

    pub async fn is_established(&self) -> bool {
        if let SessionInnerStatus::Established(_) = &*self.remote_status.read().await {
            true
        } else {
            false
        }
    }

    pub(crate) async fn vport(&self) -> Option<EndpointPair> {
        match &*self.remote_status.read().await {
            SessionInnerStatus::Established(status) => {
                if let Some(endpoint) = status.remote_external.as_ref() {
                    Some(EndpointPair::new(status.tunnel.local_endpoint(), endpoint.clone()))
                } else {
                    None
                }
            }
            _ => {
                None
            }
        }
    }

    pub async fn reset(&self) {
        let mut_status = &mut *self.remote_status.write().await;

        match mut_status {
            SessionInnerStatus::Established(_) => {},
            SessionInnerStatus::Connecting | SessionInnerStatus::Dead => {
                *mut_status = SessionInnerStatus::Connecting;
            }
        }
    }

    pub async fn as_tunnel(&self) -> NearResult<DynamicTunnel> {
        match &*self.remote_status.read().await {
            SessionInnerStatus::Established(status) => {
                Ok(status.tunnel.clone())
            }
            SessionInnerStatus::Dead => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_UNACTIVED, "session has been dead."))
            }
            SessionInnerStatus::Connecting => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_UNACTIVED, "session is connecting."))
            }
        }
    }

    pub async fn on_established(
        &self,
        tunnel: &DynamicTunnel
    ) -> NearResult<()> {

        {
            let status = &*self.remote_status.read().await;
            trace!("SessionStatus::on_established, status is {status}, tunnel: {tunnel}, self.remote_endpoint: {}", self.remote_endpoint);
        }

        if &self.remote_endpoint == tunnel.remote() {
            Ok(())
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, "endpoint unmatch"))
        }?;

        {
            let mut_status = &mut *self.remote_status.write().await;

            match mut_status {
                SessionInnerStatus::Connecting => {
                    *mut_status = 
                        SessionInnerStatus::Established(SessionEstablished {
                            remote_external: None,
                            tunnel: tunnel.clone(),
                        });
                    info!("session was {}: tunnel: {}, remote: {}.", mut_status, tunnel, self.remote_endpoint);
                    Ok(())
                }
                SessionInnerStatus::Established(c) => {
                    // ignore
                    info!("ignore: tunnel: {}, remote: {}, remote_external: {:?}, because session was established.", tunnel, self.remote_endpoint, c.remote_external);
                    Err(NearError::new(ErrorCode::NEAR_ERROR_ALREADY_EXIST, "already exist"))
                }
                SessionInnerStatus::Dead => {
                    // ignore
                    info!("ignore: tunnel: {}, because session was dead.", tunnel);
                    Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, "ignore"))
                }
            }
        }
    }

    pub async fn on_session(&self, inner: &SessionInner) {

        let task = &inner.task;
        let stack = task.as_manager().as_stack();
        let config = &stack.config().peer_c_c;

        enum NextStep {
            Connecting(u64),
            Pinging(u64),
            Stop,
        }

        let next_step = {
            let mut_remote_status = &mut *self.remote_status.write().await;

            match mut_remote_status {
                SessionInnerStatus::Connecting => 
                    NextStep::Connecting(config.ping_interval_connect.as_micros() as u64),
                SessionInnerStatus::Established(_) => 
                    NextStep::Pinging(config.ping_interval.as_micros() as u64),
                SessionInnerStatus::Dead => {
                    match task.as_remote() {
                        SessionRemoteObject::Service(_) => {
                            *mut_remote_status = SessionInnerStatus::Connecting;
                            NextStep::Connecting(config.ping_interval_connect.as_micros() as u64)
                        }
                        SessionRemoteObject::Device(_) => NextStep::Stop,
                    }
                }
            }
        };

        let now = now();
        let last_ping_time = self.last_ping_time.load(std::sync::atomic::Ordering::Acquire);
        let offline = config.offline.as_micros() as u64;

        match next_step {
            NextStep::Connecting(interval) => {
                let last_connect_time = self.last_connect_time.load(std::sync::atomic::Ordering::Acquire);
                if last_connect_time == 0 || now >= last_connect_time + interval {
                    debug!("try reconnect {} remote server.", task.as_remote_object().object_id());

                    match &inner.session_type {
                        SessionNetwork::Tcp => {
                            let _ = 
                                stack.net_manager()
                                    .connect_tcp_interface(
                                        &self.remote_endpoint,
                                        task.as_remote_object(),
                                    )
                                    .await;
                        }
                        SessionNetwork::Udp(udp) => {
                            let _ = udp.connect(&self.remote_endpoint, task.as_remote_object().clone()).await;
                        }
                    }

                    self.last_connect_time.store(now, std::sync::atomic::Ordering::Relaxed);
                }
            }
            NextStep::Pinging(interval) => {
                enum NextStep {
                    Pinging,
                    Deaded,
                    Ingore,
                }

                let next_step = {
                    if last_ping_time == 0 {
                        NextStep::Pinging
                    } else if now >= last_ping_time + interval && now < last_ping_time + offline {
                        NextStep::Pinging
                    } else if now >= last_ping_time + offline {
                        NextStep::Deaded
                    } else {
                        NextStep::Ingore
                    }
                };

                match next_step {
                    NextStep::Pinging => {
                        debug!("try ping {} remote server.", task.as_remote_object().object_id());
                        let _ = self.send_ping(stack.clone(), Some(config.ping_interval)).await;
                    },
                    NextStep::Deaded => {
                        debug!("{} disconnect.", task.as_remote_object().object_id());
                        let mut_status = &mut *self.remote_status.write().await;
                        self.last_ping_time.store(now, std::sync::atomic::Ordering::Release);

                        match mut_status {
                            SessionInnerStatus::Established(session) => {
                                stack
                                    .tunnel_manager()
                                    .close_tunnel(session.tunnel.clone());

                                task.dead(inner.session_id);

                                *mut_status = SessionInnerStatus::Dead;
                            }
                            SessionInnerStatus::Connecting => *mut_status = SessionInnerStatus::Dead,
                            SessionInnerStatus::Dead => {},
                        }
                    }
                    NextStep::Ingore => {}
                }
            }
            _ => { /* stop it, ignore */ }
        }
        /*
        let ping_interval_ms = {
            match &*self.remote_status.read().await {
                SessionInnerStatus::Established(_) => 
                    self.task.as_manager().config().ping_interval.as_micros() as u64,
                _ => { return; }
            }
        };

        let now = now();
        let last_ping_time = self.last_ping_time.load(std::sync::atomic::Ordering::Acquire);

        // let status = {
        //     self.remote_status.read().await.clone()
        // };

        // let ping_interval_ms =
        //     match status {
        //         SessionInnerStatus::Connecting(_) => self.mgr.config().ping_interval_init.as_micros() as u64,
        //         SessionInnerStatus::Established(_, _) => self.mgr.config().ping_interval.as_micros() as u64,
        //         _ => { return; }
        //     };

        // let now = now();
        // let last_ping_time = self.last_ping_time.load(std::sync::atomic::Ordering::Acquire);
        if last_ping_time == 0 || now >= last_ping_time + ping_interval_ms {
            // send ping
            let _ = self.send_ping().await;
        }

                // let now = inner.create_time.elapsed().as_millis() as u64;
                // let last_ping_time = inner.last_ping_time.load(atomic::Ordering::Acquire);
                // if last_ping_time == 0 || now < last_ping_time || now - last_ping_time >= ping_interval_ms as u64 {
                //     let _ = inner.send_ping().await;
                // }

                // // <TODO>持久化服务证明

                // let _ = inner.client_status.compare_exchange(PING_CLIENT_STATUS_BUSY, PING_CLIENT_STATUS_RUNNING, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst).unwrap();

                // // wait
                // let waiter = ping_trigger.clone();
                // let _ = async_std::io::timeout(Duration::from_millis((ping_interval_ms >> 1u32) as u64), async move {
                //     waiter.recv().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                // }).await;
        */
    }

}

impl SessionStatus {

    pub async fn on_ping_resp(
        &self,
        mut resp: StunReq,
        sequence: SequenceString,
    ) -> NearResult<()> {

        let now = now();
        self.last_resp_time.store(now, std::sync::atomic::Ordering::Release);

        match resp.stun_type() {
            StunType::PingResponse => {
                let mut_status = &mut *self.remote_status.write().await;
 
                match mut_status {
                    SessionInnerStatus::Connecting => {
                        // ignore
                        info!("{} connecting, sequence: {sequence}", self.remote_endpoint);
                    },
                    SessionInnerStatus::Established(session) => {
                        if let Some(reverse_endpoint) = resp.take_mapped_address() {
                            if let Some(remote_endpoint) = session.remote_external.as_mut() {
                                match reverse_endpoint.cmp(remote_endpoint) {
                                    std::cmp::Ordering::Equal => {},
                                    _ => { *remote_endpoint = reverse_endpoint; }
                                }
                            } else {
                                session.remote_external = Some(reverse_endpoint);
                            }
                        }

                        info!("{} established, external host: {:?}, sequence: {sequence}", self.remote_endpoint, session.remote_external);
                    }
                    SessionInnerStatus::Dead => {
                        // ignore
                        info!("{} deaded, sequence: {sequence}", self.remote_endpoint);
                    }
                }

                Ok(())
            },
            StunType::PingErrorResponse => {
                Err(resp.take_error_code().unwrap_or(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, "unknown error")))
            },
            _ => unreachable!("Not ping response")
        }?;

        self.last_ping_time.store(now, std::sync::atomic::Ordering::Release);

        Ok(())
    }

    
    pub async fn send_ping(&self, stack: Stack, interval: Option<Duration>) -> NearResult<()> {

        let (tunnel, _remote_endpoint) = {
            match &*self.remote_status.read().await {
                SessionInnerStatus::Established(c) => 
                    Ok((c.tunnel.clone(), c.remote_external.clone())),
                _ =>
                    Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "not established state"))
            }
        }?;

        let ping = StunReq::new(StunType::PingRequest);

        let (resp, sequence) = {
                CallTemplate::<StunReq>::call(
                    stack,
                    Some(tunnel),
                    RequestorMeta {
                        need_sign: true,
                        ..Default::default()
                    },
                    AnyNamedRequest::with_stun(ping),
                    interval,
                )
                .await?
        };

        info!("successfully get ping-resp: {resp}, sequence: {sequence}", );

        self.on_ping_resp(resp, sequence).await?;

        Ok(())
    }

}

struct SessionInner {
    task: Task,
    session_id: u64,
    session_type: SessionNetwork,
    session_status: SessionStatus,
    // // interface: SessionInnerType,
    // remote_endpoint: Endpoint,
    // remote_aux_endpoint: Option<Endpoint>,
    // remote_status: RwLock<SessionInnerStatus>, // 对端(SN)地址
    // // create_time: Instant,
    // last_connect_time: AtomicU64,
    // last_ping_time: AtomicU64,
    // last_resp_time: AtomicU64,

}

impl SessionInner {
    pub fn with_tcp(
        task: Task,
        remote_endpoint: Endpoint,
    ) -> NearResult<Self> {

        debug_assert!(remote_endpoint.is_tcp(), "must TCP endpoint");

        Ok(SessionInner {
            task,
            session_id: now(),
            session_type: SessionNetwork::Tcp,
            session_status: SessionStatus::new(remote_endpoint),
            // remote_endpoint,
            // remote_aux_endpoint: None,
            // remote_status: RwLock::new(SessionInnerStatus::Connecting), // 对端(SN)地址
            // // create_time: Instant::now(),
            // last_connect_time: AtomicU64::new(0),
            // last_ping_time: AtomicU64::new(now()),
            // last_resp_time: AtomicU64::new(0),
        })
    }

    pub fn with_udp(
        task: Task,
        udp: Udp,
        remote_endpoint: Endpoint,
    ) -> NearResult<Self> {

        debug_assert!(remote_endpoint.is_udp(), "must UDP endpoint");

        Ok(SessionInner {
            task,
            session_id: now(),
            session_type: SessionNetwork::Udp(udp.clone()),
            session_status: SessionStatus::new(remote_endpoint),
            // remote_endpoint,
            // remote_aux_endpoint,
            // remote_status: RwLock::new(SessionInnerStatus::Connecting), // 对端(SN)地址
            // // create_time: Instant::now(),
            // last_connect_time: AtomicU64::new(0),
            // last_ping_time: AtomicU64::new(now()),
            // last_resp_time: AtomicU64::new(0),
        })
    }

    pub async fn is_established(&self) -> bool {
        self.session_status.is_established().await
    }

    pub async fn reset(&self) {

        self.session_status.reset().await;

    }


    async fn on_established(
        &self,
        tunnel: &DynamicTunnel
    ) -> NearResult<()> {

        self.session_status.on_established(tunnel).await

    }

    pub async fn allocation_turn(
        &self,  
        sequence: &SequenceString, 
        peer_id: &ObjectId
    ) -> NearResult<()> {
        let tunnel = {
            match &*self.session_status.remote_status.read().await {
                SessionInnerStatus::Established(c) => 
                    Ok(c.tunnel.clone()),
                _ => {
                    let error_string = format!("{} not established state", self.task.0.remote.as_remote().object_id());
                    error!("{error_string}, sequence: {sequence}");
                    Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, error_string))
                }
            }
        }?;

        let (mut allocation_resp, _) = {
            CallTemplate::<StunReq>::call(
                self.task.as_manager().as_stack().clone(),
                Some(tunnel), 
                RequestorMeta {
                    to: Some(self.task.as_remote_object().object_id().clone()),
                    sequence: Some(sequence.clone()),
                    need_sign: true,
                    ..Default::default()
                },
                AnyNamedRequest::with_stun(
                    StunReq::new(crate::package::StunType::AllocationChannelRequest)
                        .set_target(Some(peer_id.clone()))
                ),
                Some(self.task.as_manager().as_stack().config().peer_c_c.call_timeout),
            )
            .await
            .map_err(| err | {
                error!("failed CallTemplate::<AllocationResp>::call with err: {err}, sequence: {sequence}");
                err
            })?
        };

        match allocation_resp.stun_type() {
            StunType::AllocationChannelErrorResponse => {
                let error = 
                    allocation_resp.take_error_code()
                        .unwrap_or(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, "unknown error"));
                Err(error)
            }
            StunType::AllocationChannelResponse => {
                let mix_hash = 
                    allocation_resp.take_mixhash()
                        .ok_or_else(|| {
                            let error_string = "missing mix-hash data";
                            error!("{error_string}, sequence: {sequence}.");
                            NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, error_string)
                        })?;
                let live_minutes = 
                    allocation_resp.take_live_minutes()
                        .ok_or_else(|| {
                            let error_string = "missing live minutes data";
                            error!("{error_string}, sequence: {sequence}.");
                            NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, error_string)
                        })?;
                let proxy_address = 
                    allocation_resp.take_proxy_address()
                        .ok_or_else(|| {
                            let error_string = "missing turn-proxy-address data";
                            error!("{error_string}, sequence: {sequence}.");
                            NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, error_string)
                        })?;

                self.task.as_manager().as_stack()
                    .turn_task()
                    .mix_hash_stubs()
                    .append(peer_id.clone(), mix_hash, live_minutes, proxy_address);

                Ok(())
            }
            StunType::AllocationChannelRequest => unreachable!("don't reach here."),
            StunType::PingRequest | StunType::PingResponse | StunType::PingErrorResponse => unreachable!("don't reach here."),
            StunType::CallRequest | StunType::CallResponse | StunType::CallErrorResponse => unreachable!("don't reach here."),
        }?;

        Ok(())
    }

    pub async fn call_peer(
        &self,  
        sequence: &SequenceString, 
        peer_id: &ObjectId
    ) -> NearResult<()> {

        let tunnel = {
            match &*self.session_status.remote_status.read().await {
                SessionInnerStatus::Established(c) => 
                    Ok(c.tunnel.clone()),
                _ => {
                    let error_string = format!("{} not established state", self.task.0.remote.as_remote().object_id());
                    error!("{error_string}, sequence: {sequence}");
                    Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, error_string))
                }
            }
        }?;

        let call_req = 
            StunReq::new(crate::package::StunType::CallRequest)
                .set_target(Some(peer_id.clone()))
                .set_fromer(Some({
                    // 携带设备信息
                    let mut local = 
                        self.task.as_manager().as_stack().cacher_manager().local();

                    let _ = 
                        std::mem::replace(
                            local.mut_body().mut_content().mut_reverse_endpoint_array(), 
                            self.task.vport_array().await
                        );

                    local
                }));

        let (mut call_resp, _) = {
                CallTemplate::<StunReq>::call(
                    self.task.as_manager().as_stack().clone(),
                    Some(tunnel), 
                    RequestorMeta {
                        to: Some(self.task.as_remote_object().object_id().clone()),
                        sequence: Some(sequence.clone()),
                        need_sign: true,
                        ..Default::default()
                    },
                    AnyNamedRequest::with_stun(call_req),
                    Some(self.task.as_manager().as_stack().config().peer_c_c.call_timeout),
                )
                .await
                .map_err(| err | {
                    error!("failed CallTemplate::<CalledResp>::call with err: {err}, sequence: {sequence}");
                    err
                })?
        };

        // call result process
        {
            info!("sequence: {sequence} call-resp: {call_resp}.",);

            match call_resp.stun_type() {
                StunType::CallErrorResponse => {
                    let error = 
                        call_resp.take_error_code()
                            .unwrap_or(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, "unknown error"));
                    Err(error)
                }
                StunType::CallResponse => {
                    let peer_info = 
                        call_resp.take_fromer().ok_or_else(|| {
                            error!("missing peer desc data sequence: {sequence}.");
                            NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing peer desc data.")
                        })?;

                    info!("[peer_id: {peer_id}, sequence: {sequence}] box endpoint-pair: {:?}", peer_info.body().content().reverse_endpoint_array());
                    let _ = 
                        self.task.as_manager().add_sn(peer_info).await
                            .map(| _ | {
                                info!("[{peer_id}] add-sn ok sequence: {sequence}");
                            })
                            .map_err(|err| {
                                error!("[{peer_id}] failed add-sn sequence: {sequence} with err: {err}");
                                err
                            })?;
                    Ok(())

                }
                StunType::CallRequest => unreachable!("don't reach here."),
                StunType::PingRequest | StunType::PingResponse | StunType::PingErrorResponse => unreachable!("don't reach here."),
                StunType::AllocationChannelRequest | StunType::AllocationChannelResponse | StunType::AllocationChannelErrorResponse => unreachable!("don't reach here."),
            }
            // let errno: ErrorCode = (call_resp.result as u16).into();


            // match errno {
            //     ErrorCode::NEAR_ERROR_SUCCESS => {
            //         let to_peer_info = call_resp.to_peer_info.ok_or_else(|| {
            //             error!("missing peer desc data sequence: {sequence}.");
            //             NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing peer desc data.")
            //         })?;

            //         info!("[peer_id: {peer_id}, sequence: {sequence}] box endpoint-pair: {:?}", to_peer_info.body().content().reverse_endpoint_array());
            //         let _ = 
            //             self.task.as_manager().add_sn(to_peer_info).await
            //                 .map(| _ | {
            //                     info!("[{peer_id}] add-sn ok sequence: {sequence}");
            //                 })
            //                 .map_err(|err| {
            //                     error!("[{peer_id}] failed add-sn sequence: {sequence} with err: {err}");
            //                     err
            //                 })?;
            //         Ok(())
            //     }
            //     _ => {
            //         Err(NearError::new(errno, format!("failed call {peer_id} with err: {errno}")))
            //     }
            // }

        }
        // let (source, _target, _) = head_ext.split();
        // let call_sequence = resp.call_sequence;
        // let to_peer_info = resp.to_peer_info.ok_or_else(|| {
        //     NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing peer desc data.")
        // })?;
        // let to_peer_id = to_peer_info.object_id().clone();
        // let errno: ErrorCode = (resp.result as u16).into();
        // log::trace!(
        //     "sequence: {} call-resp, result: {}, peer-id: {:?}, call-seq: {}.",
        //     head.sequence(),
        //     errno,
        //     to_peer_id,
        //     call_sequence
        // );

        // match errno {
        //     ErrorCode::NEAR_ERROR_SUCCESS => {
        //         CallCenterManager::get_instance()
        //             .call_result(source.requestor, to_peer_id, call_sequence, CallSessionStatus::Established)
        //             .await;

        //         let _ = self.add_sn(to_peer_info).await?;
        //     }
        //     _ => {
        //         CallCenterManager::get_instance()
        //             .call_result(source.requestor, to_peer_id, call_sequence, CallSessionStatus::Closed(errno))
        //             .await;
        //     }
        // };

        // Ok(())

        // PostMessageTrait::post_message(
        //     self.task.as_manager().as_stack(), 
        //         (
        //             Some(tunnel),
        //             RequestorMeta {
        //                 to: Some(self.task.as_remote_object().object_id().clone()),
        //                 sequence: Some(sequence.clone()),
        //                 need_sign: true,
        //                 ..Default::default()
        //             },
        //             AnyNamedRequest::with_call(call),
        //             None,
        //         )
        //     )
        //     .await?;

    }

}

impl SessionInner {

    // async fn send_ping(&self) -> NearResult<()> {

    //     let _ = 
    //         self.session_status.send_ping(
    //             self.task.as_manager().as_stack().clone(), 
    //             Some(self.task.as_manager().as_stack().config().peer_c_c.ping_interval)
    //         )
    //         .await?;

    //     // let (tunnel, _remote_endpoint) = {
    //     //     match &*self.remote_status.read().await {
    //     //         SessionInnerStatus::Established(c) => 
    //     //             Ok((c.tunnel.clone(), c.remote_external.clone())),
    //     //         _ =>
    //     //             Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "not established state"))
    //     //     }
    //     // }?;

    //     // let ping = StunReq::new(StunType::PingRequest);

    //     // let (resp, sequence) = {
    //     //         CallTemplate::<StunReq>::call(
    //     //             self.task.as_manager().as_stack().clone(),
    //     //             Some(tunnel),
    //     //             RequestorMeta {
    //     //                 need_sign: true,
    //     //                 ..Default::default()
    //     //             },
    //     //             AnyNamedRequest::with_stun(ping),
    //     //             Some(self.task.as_manager().as_stack().config().peer_c_c.ping_interval),
    //     //         )
    //     //         .await?
    //     // };

    //     // info!("successfully get ping-resp: {resp}, sequence: {sequence}", );

    //     // self.on_ping_resp(resp, sequence).await?;

    //     Ok(())
    // }

    async fn on_session(&self) {

        let _ = self.session_status.on_session(self).await;
    //     enum NextStep {
    //         Connecting(u64),
    //         Pinging(u64),
    //         Stop,
    //     }

    //     let next_step = {
    //         let mut_remote_status = &mut *self.remote_status.write().await;
    //         match mut_remote_status {
    //             SessionInnerStatus::Connecting => 
    //                 NextStep::Connecting(self.task.as_manager().config().ping_interval_connect.as_micros() as u64),
    //             SessionInnerStatus::Established(_) => 
    //                 NextStep::Pinging(self.task.as_manager().config().ping_interval.as_micros() as u64),
    //             SessionInnerStatus::Dead => {
    //                 match &self.task.as_remote() {
    //                     SessionRemoteObject::Service(_) => {
    //                         *mut_remote_status = SessionInnerStatus::Connecting;
    //                         NextStep::Connecting(self.task.as_manager().config().ping_interval_connect.as_micros() as u64)
    //                     }
    //                     SessionRemoteObject::Device(_) => NextStep::Stop,
    //                 }
    //             }
    //         }
    //     };

    //     let now = now();
    //     let last_ping_time = self.last_ping_time.load(std::sync::atomic::Ordering::Acquire);
    //     let offline = self.task.as_manager().config().offline.as_micros() as u64;

    //     match next_step {
    //         NextStep::Connecting(interval) => {
    //             let last_connect_time = self.last_connect_time.load(std::sync::atomic::Ordering::Acquire);
    //             if last_connect_time == 0 || now >= last_connect_time + interval {
    //                 debug!("try reconnect {} remote server.", self.task.as_remote_object().object_id());

    //                 match &self.session_type {
    //                     SessionNetwork::Tcp => {
    //                         let _ = 
    //                             self.task.as_manager()
    //                                 .as_stack().net_manager()
    //                                 .connect_tcp_interface(
    //                                     &self.remote_endpoint,
    //                                     self.task.as_remote_object(),
    //                                 )
    //                                 .await;
    //                     }
    //                     SessionNetwork::Udp(udp) => {
    //                         let _ = udp.connect(&self.remote_endpoint, self.task.as_remote_object().clone()).await;
    //                     }
    //                 }

    //                 self.last_connect_time.store(now, std::sync::atomic::Ordering::Relaxed);
    //             }
    //         }
    //         NextStep::Pinging(interval) => {
    //             enum NextStep {
    //                 Pinging,
    //                 Deaded,
    //                 Ingore,
    //             }

    //             let next_step = {
    //                 if last_ping_time == 0 {
    //                     NextStep::Pinging
    //                 } else if now >= last_ping_time + interval && now < last_ping_time + offline {
    //                     NextStep::Pinging
    //                 } else if now >= last_ping_time + offline {
    //                     NextStep::Deaded
    //                 } else {
    //                     NextStep::Ingore
    //                 }
    //             };

    //             match next_step {
    //                 NextStep::Pinging => {
    //                     debug!("try ping {} remote server.", self.task.as_remote_object().object_id());
    //                     let _ = self.send_ping().await;
    //                 },
    //                 NextStep::Deaded => {
    //                     debug!("{} disconnect.", self.task.as_remote_object().object_id());
    //                     let mut_status = &mut *self.remote_status.write().await;
    //                     self.last_ping_time.store(now, std::sync::atomic::Ordering::Release);

    //                     match mut_status {
    //                         SessionInnerStatus::Established(session) => {
    //                             self.task.as_manager().as_stack()
    //                                 .tunnel_manager()
    //                                 .close_tunnel(session.tunnel.clone());

    //                             self.task.dead(self.session_id);

    //                             *mut_status = SessionInnerStatus::Dead;
    //                         }
    //                         SessionInnerStatus::Connecting => *mut_status = SessionInnerStatus::Dead,
    //                         SessionInnerStatus::Dead => {},
    //                     }
    //                 }
    //                 NextStep::Ingore => {}
    //             }
    //         }
    //         _ => { /* stop it, ignore */ }
    //     }
    //     /*
    //     let ping_interval_ms = {
    //         match &*self.remote_status.read().await {
    //             SessionInnerStatus::Established(_) => 
    //                 self.task.as_manager().config().ping_interval.as_micros() as u64,
    //             _ => { return; }
    //         }
    //     };

    //     let now = now();
    //     let last_ping_time = self.last_ping_time.load(std::sync::atomic::Ordering::Acquire);

    //     // let status = {
    //     //     self.remote_status.read().await.clone()
    //     // };

    //     // let ping_interval_ms =
    //     //     match status {
    //     //         SessionInnerStatus::Connecting(_) => self.mgr.config().ping_interval_init.as_micros() as u64,
    //     //         SessionInnerStatus::Established(_, _) => self.mgr.config().ping_interval.as_micros() as u64,
    //     //         _ => { return; }
    //     //     };

    //     // let now = now();
    //     // let last_ping_time = self.last_ping_time.load(std::sync::atomic::Ordering::Acquire);
    //     if last_ping_time == 0 || now >= last_ping_time + ping_interval_ms {
    //         // send ping
    //         let _ = self.send_ping().await;
    //     }

    //             // let now = inner.create_time.elapsed().as_millis() as u64;
    //             // let last_ping_time = inner.last_ping_time.load(atomic::Ordering::Acquire);
    //             // if last_ping_time == 0 || now < last_ping_time || now - last_ping_time >= ping_interval_ms as u64 {
    //             //     let _ = inner.send_ping().await;
    //             // }

    //             // // <TODO>持久化服务证明

    //             // let _ = inner.client_status.compare_exchange(PING_CLIENT_STATUS_BUSY, PING_CLIENT_STATUS_RUNNING, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst).unwrap();

    //             // // wait
    //             // let waiter = ping_trigger.clone();
    //             // let _ = async_std::io::timeout(Duration::from_millis((ping_interval_ms >> 1u32) as u64), async move {
    //             //     waiter.recv().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    //             // }).await;
    //     */
    }
}

// impl SessionInner {
//     pub async fn on_ping_resp(
//         &self,
//         mut resp: StunReq,
//         sequence: SequenceString,
//     ) -> NearResult<()> {

//         let now = now();
//         self.last_resp_time.store(now, std::sync::atomic::Ordering::Release);

//         match resp.stun_type() {
//             StunType::PingResponse => {
//                 let mut_status = &mut *self.remote_status.write().await;
 
//                 match mut_status {
//                     SessionInnerStatus::Connecting => {
//                         // ignore
//                         info!("{} connecting, sequence: {sequence}", self.remote_endpoint);
//                     },
//                     SessionInnerStatus::Established(session) => {
//                         if let Some(reverse_endpoint) = resp.take_mapped_address() {
//                             if let Some(remote_endpoint) = session.remote_external.as_mut() {
//                                 match reverse_endpoint.cmp(remote_endpoint) {
//                                     std::cmp::Ordering::Equal => {},
//                                     _ => { *remote_endpoint = reverse_endpoint; }
//                                 }
//                             } else {
//                                 session.remote_external = Some(reverse_endpoint);
//                             }
//                         }

//                         info!("{} established, external host: {:?}, sequence: {sequence}", self.remote_endpoint, session.remote_external);
//                     }
//                     SessionInnerStatus::Dead => {
//                         // ignore
//                         info!("{} deaded, sequence: {sequence}", self.remote_endpoint);
//                     }
//                 }

//                 Ok(())
//             },
//             StunType::PingErrorResponse => {
//                 Err(resp.take_error_code().unwrap_or(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, "unknown error")))
//             },
//             _ => unreachable!("Not ping response")
//         }?;

//         self.last_ping_time.store(now, std::sync::atomic::Ordering::Release);

//         Ok(())

//         // let now = self.create_time.elapsed().as_millis() as u64;
//         // self.last_resp_time.store(now, atomic::Ordering::Release);

//         // let mut rto = 0;
//         // let mut is_handled = false;

//         // let active_session_index = self.active_session_index.load(atomic::Ordering::Acquire) as usize;
//         // let sessions = self.sessions.read().unwrap();
//         // let try_session = sessions.get(active_session_index);
//         // let mut new_endpoint = UpdateOuterResult::None;
//         // if let Some(s) = try_session {
//         //     let r = s.on_ping_resp(resp, from, from_interface.clone(), now, &mut rto, &mut is_handled);
//         //     new_endpoint = std::cmp::max(r, new_endpoint);
//         // }

//         // if !is_handled {
//         //     let mut index = 0;
//         //     for session in (*sessions).as_slice() {
//         //         let r = session.on_ping_resp(resp, from, from_interface.clone(), now, &mut rto, &mut is_handled);
//         //         new_endpoint = std::cmp::max(r, new_endpoint);
//         //         if is_handled {
//         //             let _ = self.active_session_index.compare_exchange(std::u32::MAX, index, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst);
//         //             break;
//         //         }
//         //         index += 1;
//         //     }
//         // }

//         // self.update_status();

//         // self.contract.on_ping_resp(resp, rto);

//         // let is_resend_immdiate = if resp.result == BuckyErrorCode::NotFound.as_u8() {
//         //     // 要更新desc
//         //     let _ = self.last_update_seq.compare_exchange(0, 1, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst);
//         //     true
//         // } else {
//         //     let _ = self.last_update_seq.compare_exchange(resp.seq.value(), 0, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst);
//         //     false
//         // };

//         // (new_endpoint, is_resend_immdiate)

//     }
// }

#[async_trait::async_trait]
impl PostMessageTrait<(ObjectId, SequenceString, PackageDataSet)> for SessionInner {

    type R = ();

    async fn post_message(
        &self, 
        context: (ObjectId, SequenceString, PackageDataSet)
    ) -> NearResult<Self::R> {

        let (target, sequence, package) = context;

        trace!("SessionInner::post_message, target: {}, sequence: {}", target, sequence);

        let tunnel = 
            self.session_status.as_tunnel()
                .await
                .map_err(| err | {
                    error!("failed get tunnel with err: {err}");
                    err
                })?;

        self.task.as_manager().as_stack()
            .tunnel_manager()
            .post_message((tunnel, sequence, package))
            .await
    }
}

enum TaskStateImpl {
    Connecting(StateWaiter),
    Actived(u8 /* actived_counter */),
    Deaded,
}

impl std::fmt::Display for TaskStateImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
                Self::Connecting(_) => "connecting".to_owned(),
                Self::Actived(counter) => format!("actived({counter})"),
                Self::Deaded => "deaded".to_owned(),
            }
        )
    }
}

struct TaskInner {
    mgr: PingManager,

    remote: SessionRemoteObject,

    sessions: Vec<Arc<SessionInner>>,

    state: Mutex<TaskStateImpl>,

    network_access_type: AtomicU8,
}

impl TaskInner {
    async fn on_process(&self) -> NearResult<()> {

        trace!("{} will ping process.", 
            self.remote.as_remote().object_id(),
        );

        let (check_network_session, futs) = {
            let mut chks = vec![];
            let mut futs = vec![];

            for iter in self.sessions.iter() {
                chks.push(!iter.is_established().await);
                futs.push(iter.on_session());
            }

            (chks, futs)
        };

        let _ = futures::future::join_all(futs).await;

        for (idx, check_network) in check_network_session.iter().enumerate() {
            if !check_network {
                continue;
            }

            if let Some(session) = self.sessions.get(idx) {
                let port_pair = 
                    session.session_status.vport().await.ok_or_else(|| {
                        error!("failed get main port");
                        NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "not found main external port")
                    })?;

                self.mgr.report_network_endpoint(port_pair.remote()).await;
            }
        }

        let network_access_type = self.mgr.network_access_type().await;
        if self.network_access_type.load(std::sync::atomic::Ordering::SeqCst) == network_access_type as u8 {
            let network_access_type = self.mgr.network_access_type().await;
            if self.network_access_type.compare_exchange(
                    NetworkAccessType::Unknown as u8,
                    network_access_type as u8,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst
                )
                .is_ok() {
                info!("change network access type: {network_access_type}");
            }
        }

        Ok(())
    }

}

impl TaskInner {
    async fn on_established(
        &self,
        tunnel: DynamicTunnel
    ) -> NearResult<()> {

        let mut sessions_iter = self.sessions.iter();

        loop {
            if let Some(session_next) = sessions_iter.next() {
                match session_next.on_established(&tunnel).await {
                    Ok(_) => break(Ok(())),
                    Err(err) => {
                        match err.errno() {
                            ErrorCode::NEAR_ERROR_ALREADY_EXIST => break(Err(err)),
                            _ => continue
                        }
                    }
                }
            } else {
                break(Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, "not found matched session.")))
            }
        }

    }

    // pub async fn on_ping_resp(
    //     &self,
    //     resp: PingResp,
    // ) -> NearResult<()> {

    //     let session = 
    //         self.sessions.iter().find(| it | {
    //             it.session_id == resp.session_id
    //         })
    //         .cloned()
    //         .ok_or_else(|| {
    //             let error_string = format!("{} not found.", resp.session_id);
    //             warn!("{error_string}");
    //             NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
    //         })?;

    //     session.on_ping_resp(resp).await

    // }
}

unsafe impl Sync for TaskInner {}
unsafe impl Send for TaskInner {}

#[derive(Clone)]
pub struct Task(Arc<TaskInner>);

impl Task {

    async fn with_capacity(
        mgr: PingManager, 
        remote: DeviceObject, 
        include_tcp: bool
    ) -> NearResult<Self> {

        let remote_type_codec = remote.object_id().object_type_code()?;

        let task = Self(Arc::new(TaskInner {
            mgr: mgr.clone(),
            remote: remote.try_into()?,
            sessions: vec![],
            // session_actived: AtomicU8::new(0),
            state: Mutex::new(TaskStateImpl::Connecting(StateWaiter::new())),
            network_access_type: AtomicU8::new(NetworkAccessType::Unknown as u8),
        }));

        let mut sessions = vec![];

        // add udp session
        let vports = mgr.vports().await;
        enum PortAccessType<'a> {
            TCP(&'a Endpoint),
            UDP(&'a Endpoint),
        }

        let port = {
            match remote_type_codec {
                ObjectTypeCode::Device(v) if v == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => {
                    let udp_endpoints: Vec<&Endpoint> = task.as_remote_object().body().content().reverse_endpoint_array().iter().map(| ep | ep.remote()).filter(| ep | ep.is_udp() ).collect();

                    Ok(PortAccessType::UDP(
                        udp_endpoints.get(0).ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_DONOT_EXIST, "not found reverse endpoint."))?, 
                    ))
                }
                ObjectTypeCode::Service(v) if v == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_COTURN_MINER as u8 => {
                    let udp_endpoints: Vec<&Endpoint> = task.as_remote_object().body().content().endpoints().iter().filter(| ep | ep.is_udp() ).collect();
                    let tcp_endpoints: Vec<&Endpoint> = task.as_remote_object().body().content().endpoints().iter().filter(| ep | ep.is_tcp() ).collect();

                    if udp_endpoints.len() != 0 {
                        let main_ep = udp_endpoints.get(0).map(| &endpoint | endpoint).unwrap();
//                        let aux_ep = udp_endpoints.get(1).map(| &endpoint | endpoint);
        
                        Ok(PortAccessType::UDP(main_ep))
                    } else if tcp_endpoints.len() != 0 {
                        let main_ep = tcp_endpoints.get(0).map(| &endpoint | endpoint).unwrap();
                        Ok(PortAccessType::TCP(main_ep))
                    } else {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing main sn endpoint"))
                    }
                }
                _ => { Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, "ignore")) }
            }

            // let udp_endpoints: Vec<&Endpoint> = task.as_remote_object().body().content().endpoints().iter().filter(| ep | ep.is_udp() ).collect();
            // let tcp_endpoints: Vec<&Endpoint> = task.as_remote_object().body().content().endpoints().iter().filter(| ep | ep.is_tcp() ).collect();
            // if udp_endpoints.len() != 0 {
            //     let main_ep = udp_endpoints.get(0).map(| &endpoint | endpoint).unwrap();
            //     let aux_ep = udp_endpoints.get(1).map(| &endpoint | endpoint);

            //     Ok(PortAccessType::UDP(main_ep, aux_ep))
            // } else if tcp_endpoints.len() != 0 {
            //     let main_ep = tcp_endpoints.get(0).map(| &endpoint | endpoint).unwrap();
            //     Ok(PortAccessType::TCP(main_ep))
            // } else {
            //     Err(NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing main sn endpoint"))
            // }
        }?;

        match port {
            PortAccessType::TCP(ep) => {
                if include_tcp {
                    match SessionInner::with_tcp(task.clone(), ep.clone()) {
                        Ok(session) => {
                            sessions.push(Arc::new(session));
                        }
                        Err(e) => {
                            warn!("failed connect in session with err: {e}");
                        }
                    }
                } else {
                    info!("uninclude tcp");
                }
            }
            PortAccessType::UDP(ep) => {
                for vport in vports.iter() {
                    match SessionInner::with_udp(task.clone(), vport.clone(), ep.clone()) {
                        Ok(session) => {
                            sessions.push(Arc::new(session));
                        }
                        Err(e) => {
                            warn!("failed connect in session with err: {e}");
                        }
                    }
                }
            }
        };

        std::mem::swap(&mut unsafe { &mut *(Arc::as_ptr(&task.0) as *mut TaskInner) }.sessions, &mut sessions);

        Ok(task)

    }

    // async fn with_box(mgr: PingManager, remote: DeviceObject) -> NearResult<Self> {
    //     let task = Self(Arc::new(TaskInner {
    //         mgr: mgr.clone(),
    //         remote: remote.try_into()?,
    //         sessions: vec![],
    //         session_actived: AtomicU8::new(0),
    //         state: Mutex::new(TaskStateImpl::Connecting(StateWaiter::new())),
    //         network_access_type: RwLock::new(NetworkAccessType::None),
    //     }));

    //     let mut sessions = vec![];

    //     // add udp session
    //     let vports = mgr.vports().await;
    //     for endpoint in task.as_remote_object().body().content().reverse_endpoint_array().iter().map(| pair | pair.remote() ).filter(| &ep | ep.is_udp() ) {
    //         for vport in vports.iter() {
    //             match SessionInner::with_udp(task.clone(), vport.clone(), endpoint.clone(), aux) {
    //                 Ok(session) => sessions.push(Arc::new(session)),
    //                 Err(e) => {
    //                     warn!("failed connect in session with err: {e}");
    //                 }
    //             }
    //         }
    //     }

    //     std::mem::swap(&mut unsafe { &mut *(Arc::as_ptr(&task.0) as *mut TaskInner) }.sessions, &mut sessions);

    //     Ok(task)
    // }

    pub(crate) async fn new(mgr: PingManager, remote: DeviceObject) -> NearResult<Self> {

        let codec = remote.object_id().object_type_code()?;
        let task = 
            match codec {
                ObjectTypeCode::Service(sub_codec) if sub_codec == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_COTURN_MINER as u8 => {
                    Task::with_capacity(mgr, remote, true).await
                }
                ObjectTypeCode::Device(sub_codec) if  sub_codec == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => {
                    Task::with_capacity(mgr, remote, false).await
                }
                _ => {
                    Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("device object codec must service object or device object, expr: {}.", codec)))
                }
            }?;

        Ok(task)
    }

    pub(crate) async fn reset(&self, _endpoint: Option<Endpoint>) -> NearResult<()> {
        let sessions = self.0.sessions.clone();

        if sessions.len() <= 0 {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "not found session"));
        }

        let mut futs = vec![];
        for session in sessions.iter() {
            futs.push(session.reset());
        }

        let _ = futures::future::join_all(futs).await;

        Ok(())
    }

    pub(crate) async fn on_process(&self) -> NearResult<()> {
        self.0.on_process().await
    }

    pub(crate) fn as_manager(&self) -> &PingManager {
        &self.0.mgr
    }

    pub(crate) fn as_remote_object(&self) -> &DeviceObject {
        self.0.remote.as_remote()
    }

    pub(self) fn as_remote(&self) -> &SessionRemoteObject {
        &self.0.remote
    }

    #[inline]
    #[allow(unused)]
    pub async fn network_access_type(&self) -> NetworkAccessType {
        self.0.network_access_type.load(std::sync::atomic::Ordering::SeqCst).try_into().unwrap()
    }

    #[inline]
    #[allow(unused)]
    pub async fn change_network_access_type(&self, network_access_type: NetworkAccessType) {
        self.0.network_access_type.store(network_access_type as u8, std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) async fn vport_array(&self) -> Vec<EndpointPair> {
        let mut array = vec![];

        for session in self.0.sessions.iter() {
            if let Some(pair) = session.session_status.vport().await {
                array.push(pair);
            }
        }

        array
    }

    pub async fn call_peer(
        &self, 
        sequence: &SequenceString, 
        remote: &ObjectId
    ) -> NearResult<()> {

        let random_index = now() as usize % self.0.sessions.len();

        self.0.sessions.get(random_index)
            .ok_or_else(|| {
                let error_string = format!("not found task session, index: {}, len: {}", random_index, self.0.sessions.len());
                error!("{error_string}, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
            })?
            .call_peer(sequence, remote)
            .await
    }

    pub async fn allocation_turn(
        &self,
        sequence: &SequenceString,
        peer_id: &ObjectId
    ) -> NearResult<()> {

        let random_index = now() as usize % self.0.sessions.len();

        self.0.sessions.get(random_index)
            .ok_or_else(|| {
                let error_string = format!("not found task session, index: {}, len: {}", random_index, self.0.sessions.len());
                error!("{error_string}, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
            })?
            .allocation_turn(sequence, peer_id)
            .await
    }
}

impl Task {

    pub(crate) fn is_active(&self) -> bool {
        let state = &*self.0.state.lock().unwrap();
        match state {
            TaskStateImpl::Connecting(_) => false,
            TaskStateImpl::Actived(_) => true,
            TaskStateImpl::Deaded => false,
        }
    }

    pub(in self) fn active(&self) {
        let waker = {
            let state = &mut *self.0.state.lock().unwrap();
            match state {
                TaskStateImpl::Actived(counter) => {
                    *counter += 1;
                    None
                }
                TaskStateImpl::Connecting(waiter) => {
                    let waker = waiter.transfer();

                    *state = TaskStateImpl::Actived(1);

                    Some(waker)
                }
                TaskStateImpl::Deaded => unreachable!("don't reach here.")
            }
        };

        if let Some(waker) = waker {
            waker.wake();
        }
    }

    pub(in self) fn dead(&self, session_id: u64) {
        // if self.0.session_actived.load(std::sync::atomic::Ordering::SeqCst) == 0 {
        //     return;
        // }

        let mut finder = false;

        for session in self.0.sessions.iter() {
            if session.session_id == session_id {
                finder = true;
                break;
            }
        }

        if finder {
            let state = &mut *self.0.state.lock().unwrap();

            match state {
                TaskStateImpl::Actived(counter) => {
                    *counter -= 1;
                    if *counter == 0 {
                        if self.0.remote.is_service() {
                            *state = TaskStateImpl::Connecting(StateWaiter::new());
                        } else {
                            *state = TaskStateImpl::Deaded;
                        }
                    }
                }
                TaskStateImpl::Connecting(_) | TaskStateImpl::Deaded => unreachable!("don't reach here.")
            }

            info!("Task::dead: state: {state}");
        }
    }

    pub(crate) async fn wait_online(&self, timeout: Option<Duration>) -> bool {

        let (waiter, online) = {
            let state = &mut *self.0.state.lock().unwrap();
            match state {
                TaskStateImpl::Connecting(waiter) => (Some(waiter.new_waiter()), false),
                TaskStateImpl::Actived(_) => (None, true),
                TaskStateImpl::Deaded => (None, false),
            }
        };

        if online {
            true
        } else if let Some(waiter) = waiter {
            if let Some(timeout) = timeout {
                async_std::future::timeout(
                    timeout, 
                    StateWaiter::wait(waiter, || self.is_active())
                )
                .await
                .unwrap_or(false)
            } else {
                StateWaiter::wait(waiter, || self.is_active()).await
            }
        } else {
            false
        }

    }

}

#[async_trait::async_trait]
impl PackageEstablishedTrait for Task {
    async fn on_established(
        &self,
        tunnel: DynamicTunnel
    ) {
        if let Ok(_) = self.0.on_established(tunnel).await {
            // if self.0.session_actived.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1 == 1 {
            self.active();
            // }
            // info!("Task::on_established: session_actived is {}", self.0.session_actived.load(std::sync::atomic::Ordering::SeqCst));
        }
    }
}

// impl Task {
//     pub async fn on_ping_resp(
//         &self,
//         resp: PingResp,
//     ) -> NearResult<()> {
//         self.0.on_ping_resp(resp).await
//     }
// }

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_info = String::default();

        self.0.sessions.iter()
            .for_each(| session | {
                let _ = debug_info.write_fmt(format_args!("{}, ", session.session_id));
            });

        write!(f, "{debug_info}")
    }
}

#[async_trait::async_trait]
impl PostMessageTrait<(ObjectId, SequenceString, PackageDataSet)> for Task {

    type R = ();

    async fn post_message(
        &self, 
        context: (ObjectId, SequenceString, PackageDataSet)
    ) -> NearResult<Self::R> {

        self.get_session(Some(self.as_manager().config().call_timeout))
            .await?
            .post_message(context)
            .await

    }
}

impl Task {
    async fn get_session(&self, timeout: Option<Duration>) -> NearResult<Arc<SessionInner>> {
        let sessions = self.0.sessions.clone();

        let sessions_count = sessions.len();
        if sessions_count == 0 {
            warn!("no found sessions");
            return Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "no found sessions"));
        }

        if self.wait_online(timeout).await {
            Ok(())
        } else {
            // // timeout
            // let network_access_type = self.network_access_type().await;

            // match mut_network_access_type {
            //     NetworkAccessType::Unknown => unreachable!("don't reach here."),
            //     NetworkAccessType::NAT => {
            //         // timeout, it may not be NAT. here will charget Symmetric type.
            //         *mut_network_access_type = NetworkAccessType::Symmetric;
            //         Err(NearError::new(ErrorCode::NEAR_ERROR_RETRY, format!("{} need retry.", self.0.remote.as_remote().object_id())))
            //     }
            //     NetworkAccessType::Symmetric => {
            //         Err(NearError::new(ErrorCode::NEAR_ERROR_RETRY, format!("retry")))
            //     }
            // }
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNACTIVED, format!("{} unactived.", self.0.remote.as_remote().object_id())))
        }?;

        let sessions_ref = sessions.as_slice();
        let mut index = now() as usize % sessions_count;
        let orig_index = index;

        let session = loop {
            let session = &sessions_ref[index];
            if session.is_established().await {
                break (Some(session));
            } else {
                index += 1;

                if index == sessions_count {
                    index = 0;
                }
                if index == orig_index {
                    break (None);
                }
            }
        };

        if let Some(session) = session {
            Ok(session.clone())
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "not found session"))
        }
    }
}
