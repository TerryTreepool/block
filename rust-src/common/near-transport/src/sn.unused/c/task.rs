
use std::{fmt::Write, sync::{atomic::AtomicU64, Arc}};
use async_std::sync::RwLock;

use log::{debug, error, info, trace, warn};
use near_base::{now, DeviceObject, DeviceObjectSubCode, Endpoint, EndpointPair, ErrorCode, NearError, NearResult, ObjectId, ObjectTypeCode, Sequence, ServiceObjectSubCode};

use crate::{h::OnBuildPackage, network::Udp, 
        package::{AnyNamedRequest, CallReq, Data, Ping, PingResp }, 
        process::PackageEstablishedTrait, stack::BuildPackageV1, 
        tunnel::{DynamicTunnel, PostMessageTrait}, HeaderMeta 
    };

use super::{call::CallResultRef, ping::PingManager};

enum SessionInnerStatus {
    Connecting,
    Established(SessionEstablished),
    Dead,
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
            ObjectTypeCode::Service(o) if o == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_SN_MINER as u8 => 
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
}

struct SessionInner {
    task: Task,
    session_id: u64,
    session_type: SessionNetwork,
    // interface: SessionInnerType,
    remote_endpoint: Endpoint,
    remote_status: RwLock<SessionInnerStatus>, // 对端(SN)地址
    // create_time: Instant,
    last_ping_sequence: Sequence,
    last_connect_time: AtomicU64,
    last_ping_time: AtomicU64,
    last_resp_time: AtomicU64,

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
            remote_endpoint,
            remote_status: RwLock::new(SessionInnerStatus::Connecting), // 对端(SN)地址
            // create_time: Instant::now(),
            last_ping_sequence: Sequence::random(),
            last_connect_time: AtomicU64::new(0),
            last_ping_time: AtomicU64::new(0),
            last_resp_time: AtomicU64::new(0),
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
            remote_endpoint,
            remote_status: RwLock::new(SessionInnerStatus::Connecting), // 对端(SN)地址
            // create_time: Instant::now(),
            last_ping_sequence: Sequence::random(),
            last_connect_time: AtomicU64::new(0),
            last_ping_time: AtomicU64::new(0),
            last_resp_time: AtomicU64::new(0),
        })
    }

    pub async fn is_established(&self) -> bool {
        if let SessionInnerStatus::Established(_) = &*self.remote_status.read().await {
            true
        } else {
            false
        }
    }

    pub async fn reset(&self) -> NearResult<()> {

        {
            let mut_status = &mut *self.remote_status.write().await;

            match mut_status {
                SessionInnerStatus::Established(_) => Err(NearError::new(ErrorCode::NEAR_ERROR_ALREADY_EXIST, "already connecting.")),
                SessionInnerStatus::Connecting | SessionInnerStatus::Dead => {
                    *mut_status = SessionInnerStatus::Connecting;
                    Ok(())
                }
            }?;
        }

        // match &self.session_type {
        //     SessionNetwork::Tcp => {
        //         let _ = self.task.as_manager().as_stack().net_manager().connect_tcp_interface(&self.remote_endpoint, self.remote.as_remote()).await?;
        //     }
        //     SessionNetwork::Udp(udp) => {
        //         let _ = udp.connect(&self.remote_endpoint, self.remote.as_remote().clone()).await?;
        //     }
        // }
        
        Ok(())
    }

    // pub(crate) async fn reset(&self) -> NearResult<()> {
    //     let mut_status = &mut *self.remote_status.write().await;

    //     match mut_status {
    //         SessionInnerStatus::Connecting | SessionInnerStatus::Established(_) => Ok(()),
    //         SessionInnerStatus::Dead => {
    //             Ok(())
    //         }
    //     }
    // }

    async fn on_established(
        &self,
        tunnel: &DynamicTunnel
    ) -> NearResult<()> {
        {
            let mut_status = &mut *self.remote_status.write().await;

            match mut_status {
                SessionInnerStatus::Connecting => {
                    if &self.remote_endpoint == tunnel.remote() {
                        *mut_status =
                            SessionInnerStatus::Established(SessionEstablished {
                                remote_external: None,
                                tunnel: tunnel.clone(),
                            });
                        Ok(())
                    } else {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, "endpoint unmatch"))
                    }
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

    pub async fn call_peer(&self, peer_id: &ObjectId) -> NearResult<CallResultRef> {

        let (tunnel, _remote_endpoint) = {
            match &*self.remote_status.read().await {
                SessionInnerStatus::Established(c) => 
                    Ok((c.tunnel.clone(), c.remote_external.clone())),
                _ =>
                    Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "not established state"))
            }
        }?;

        let call = CallReq {
            session_id: self.session_id,
            call_sequence: self.last_ping_sequence.generate().into_value(),
            to_peer_id: peer_id.clone(),
            // fromer: Some(self.task.as_manager().as_stack().cacher_manager().local()),
            fromer: Some({
                // 携带设备信息
                let mut local = 
                    self.task.as_manager().as_stack().cacher_manager().local();

                let _ = 
                    std::mem::replace(
                        local.mut_body().mut_content().mut_reverse_endpoint_array(), 
                        self.task.vport_array().await
                    );

                local
            }),
            call_time: now(),
        };

        let f = 
            CallResultRef::new(
                    self.task.as_remote_object().object_id().clone(), 
                    peer_id.clone(), 
                    call.call_sequence
                )
                .map_err(| err | {
                    error!("failed CallResultRef::new() with err: {err}");
                    err
                })?;

        if let Ok((sequence, package)) =
            self.task.as_manager()
                .as_stack()
                .build_package(
                    BuildPackageV1 {
                            remote: Some(self.task.as_remote_object().object_id().clone()),
                            body: AnyNamedRequest::with_call(call),
                            need_sign: true,
                            ..Default::default()
                        },
                )
                .await
                .map_err(| err | {
                    error!("failed build CallReq package to {} with {}", tunnel, err);
                    err
                }) {
                let _ =
                    self.task.as_manager().as_stack()
                        .tunnel_manager()
                        .post_message((tunnel.clone(), sequence.clone(), package))
                        .await
                        .map(|_| info!("succeed send CallReq pacage to {}, sequence: {}", tunnel, sequence) )
                        .map_err(| err | error!("failed send CallReq to {} with {}, sequence: {}", tunnel, err, sequence) );
            }

        Ok(f)
    }

}

impl SessionInner {

    async fn send_ping(&self) -> NearResult<()> {

        let (tunnel, _remote_endpoint) = {
            match &*self.remote_status.read().await {
                SessionInnerStatus::Established(c) => 
                    Ok((c.tunnel.clone(), c.remote_external.clone())),
                _ =>
                    Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "not established state"))
            }
        }?;

        let vport_array = self.task.vport_array().await;

        let now = now();
        let ping = Ping {

            session_id: self.session_id,
            send_time: now,
            ping_sequence: self.last_ping_sequence.generate().into_value(),
            peer_id: self.task.as_manager().as_stack().local_device_id().clone(),
            // 发送者设备信息
            peer_info: Some({
                let mut local = 
                    self.task.as_manager().as_stack().cacher_manager().local();

                let _ = 
                    std::mem::replace(
                        local.mut_body().mut_content().mut_reverse_endpoint_array(), 
                        vport_array
                    );

                local
            }),
            nonce: now.to_string(),
        };

        if let Ok((sequence, package)) =
            self.task.as_manager().as_stack()
                .build_package(
                    BuildPackageV1 {
                            remote: Some(self.task.as_remote_object().object_id().clone()),
                            body: AnyNamedRequest::with_ping(ping),
                            need_sign: true,
                            ..Default::default()
                        },
                )
                .await
                .map_err(| err | {
                    error!("failed build Ping package to {} with {}", tunnel, err);
                    err
                }) {
                let _ =
                    self.task.as_manager().as_stack()
                        .tunnel_manager()
                        .post_message((tunnel.clone(), sequence, package))
                        .await
                        .map(|_| info!("succeed send PING pacage to {}", tunnel) )
                        .map_err(| err | error!("failed send data to {} with {}", tunnel, err) );
            }


        Ok(())
    }

    async fn on_session(&self) {

        enum NextStep {
            Connecting(u64),
            Pinging(u64),
            Stop,
        }

        let next_step = {
            let mut_remote_status = &mut *self.remote_status.write().await;
            match mut_remote_status {
                SessionInnerStatus::Connecting => 
                    NextStep::Connecting(self.task.as_manager().config().ping_interval_connect.as_micros() as u64),
                SessionInnerStatus::Established(_) => 
                    NextStep::Pinging(self.task.as_manager().config().ping_interval.as_micros() as u64),
                SessionInnerStatus::Dead => {
                    match &self.task.as_remote() {
                        SessionRemoteObject::Service(_) => {
                            *mut_remote_status = SessionInnerStatus::Connecting;
                            NextStep::Connecting(self.task.as_manager().config().ping_interval_connect.as_micros() as u64)
                        }
                        SessionRemoteObject::Device(_) => NextStep::Stop,
                    }
                }
            }
        };

        let now = now();
        let last_ping_time = self.last_ping_time.load(std::sync::atomic::Ordering::Acquire);
        let offline = self.task.as_manager().config().offline.as_micros() as u64;

        match next_step {
            NextStep::Connecting(interval) => {
                let last_connect_time = self.last_connect_time.load(std::sync::atomic::Ordering::Acquire);
                if last_connect_time == 0 || now >= last_connect_time + interval {
                    debug!("try reconnect {} remote server.", self.task.as_remote_object().object_id());

                    match &self.session_type {
                        SessionNetwork::Tcp => {
                            let _ = 
                                self.task.as_manager()
                                    .as_stack().net_manager()
                                    .connect_tcp_interface(
                                        &self.remote_endpoint,
                                        self.task.as_remote_object(),
                                    )
                                    .await;
                        }
                        SessionNetwork::Udp(udp) => {
                            let _ = udp.connect(&self.remote_endpoint, self.task.as_remote_object().clone()).await;
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
                        debug!("try ping {} remote server.", self.task.as_remote_object().object_id());
                        let _ = self.send_ping().await;
                    },
                    NextStep::Deaded => {
                        debug!("{} disconnect.", self.task.as_remote_object().object_id());
                        let mut_status = &mut *self.remote_status.write().await;
                        self.last_ping_time.store(0, std::sync::atomic::Ordering::Release);

                        match mut_status {
                            SessionInnerStatus::Established(session) => {
                                self.task.as_manager().as_stack()
                                    .tunnel_manager()
                                    .close_tunnel(session.tunnel.clone());
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

impl SessionInner {
    pub async fn on_ping_resp(
        &self,
        resp: PingResp,
    ) -> NearResult<()> {
        // check resp sequence
        {
            let ping_sequence = self.last_ping_sequence.into_value().into_value();
            if resp.ping_sequence == ping_sequence {
                Ok(())
            } else {
                let error_string = format!("Failed to verify sequence, inconsistent response and request, sequence: got={}, expr={}", resp.ping_sequence, ping_sequence);
                info!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, error_string))
            }
        }?;

        let now = now();
        self.last_resp_time.store(now, std::sync::atomic::Ordering::Release);

        {
            let mut_status = &mut *self.remote_status.write().await;
 
            match mut_status {
                SessionInnerStatus::Connecting => {
                    // ignore
                    info!("{} connecting", self.remote_endpoint);
                },
                SessionInnerStatus::Established(session) => {
                    if let Some(reverse_endpoint) = resp.reverse_endpoint {
                        if let Some(remote_endpoint) = session.remote_external.as_mut() {
                            match reverse_endpoint.cmp(remote_endpoint) {
                                std::cmp::Ordering::Equal => {},
                                _ => { *remote_endpoint = reverse_endpoint; }
                            }
                        } else {
                            session.remote_external = Some(reverse_endpoint);
                        }
                    }

                    // update local
                    // self.task.as_manager().as_stack()
                    //     .cacher_manager()
                    //     .local()
                    info!("{} established, external host: {:?}", self.remote_endpoint, session.remote_external);
                }
                SessionInnerStatus::Dead => {
                    // ignore
                    info!("{} deaded", self.remote_endpoint);
                }
            }
        }
        self.last_ping_time.store(now, std::sync::atomic::Ordering::Release);

        Ok(())


        // let now = self.create_time.elapsed().as_millis() as u64;
        // self.last_resp_time.store(now, atomic::Ordering::Release);

        // let mut rto = 0;
        // let mut is_handled = false;

        // let active_session_index = self.active_session_index.load(atomic::Ordering::Acquire) as usize;
        // let sessions = self.sessions.read().unwrap();
        // let try_session = sessions.get(active_session_index);
        // let mut new_endpoint = UpdateOuterResult::None;
        // if let Some(s) = try_session {
        //     let r = s.on_ping_resp(resp, from, from_interface.clone(), now, &mut rto, &mut is_handled);
        //     new_endpoint = std::cmp::max(r, new_endpoint);
        // }

        // if !is_handled {
        //     let mut index = 0;
        //     for session in (*sessions).as_slice() {
        //         let r = session.on_ping_resp(resp, from, from_interface.clone(), now, &mut rto, &mut is_handled);
        //         new_endpoint = std::cmp::max(r, new_endpoint);
        //         if is_handled {
        //             let _ = self.active_session_index.compare_exchange(std::u32::MAX, index, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst);
        //             break;
        //         }
        //         index += 1;
        //     }
        // }

        // self.update_status();

        // self.contract.on_ping_resp(resp, rto);

        // let is_resend_immdiate = if resp.result == BuckyErrorCode::NotFound.as_u8() {
        //     // 要更新desc
        //     let _ = self.last_update_seq.compare_exchange(0, 1, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst);
        //     true
        // } else {
        //     let _ = self.last_update_seq.compare_exchange(resp.seq.value(), 0, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst);
        //     false
        // };

        // (new_endpoint, is_resend_immdiate)

    }
}

#[async_trait::async_trait]
impl PostMessageTrait<(HeaderMeta, Data)> for SessionInner {

    async fn post_message(&self, context: (HeaderMeta, Data)) -> NearResult<()> {

        let (header_meta, data) = context;

        trace!("SessionInner::post_message, header_meta: {}", header_meta);

        let tunnel = 
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
            }?;

        self.task.as_manager().as_stack()
            .tunnel_manager()
            .post_message((tunnel, header_meta, data))
            .await
    }
}


struct TaskInner {
    mgr: PingManager,

    remote: SessionRemoteObject,

    // remote_id: DeviceId,
    // remote: DeviceObject,

    // actived: AtomicBool,
    // sn_server_status: RwLock<SNStatusInner>,

    // sender: async_std::channel::Sender<()>,
    // recver: async_std::channel::Receiver<()>,

    sessions: Vec<Arc<SessionInner>>,
    // active_session_index: AtomicU32,

    // ping_status: RwLock<PingState>,
    // task_status: AtomicU8,
    // sn_status: AtomicU8,

    // last_ping_time: AtomicU64,
    // last_resp_time: AtomicU64,
    // last_update_seq: AtomicU32,

    // seq_genarator: TempSeqGenerator,

    // contract: Contract,
}

impl TaskInner {
    async fn on_process(&self) -> NearResult<()> {

        trace!("{} will ping process.", self.remote.as_remote().object_id());

        let mut session_fut = vec![];
        for session in self.sessions.iter() {
            session_fut.push(
                session.on_session()
            );
        }

        let _ = futures::future::join_all(session_fut).await;

        Ok(())
    }

}

impl TaskInner {
    async fn on_established(
        &self,
        tunnel: DynamicTunnel
    ) {
        for session in self.sessions.iter() {
            if let Ok(_) = session.on_established(&tunnel).await {
                break;
            }
        }
    }

    pub async fn on_ping_resp(
        &self,
        resp: PingResp,
    ) -> NearResult<()> {

        let session = 
            self.sessions.iter().find(| it | {
                it.session_id == resp.session_id
            })
            .cloned()
            .ok_or_else(|| {
                let error_string = format!("{} not found.", resp.session_id);
                warn!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
            })?;

        session.on_ping_resp(resp).await

    }
}

unsafe impl Sync for TaskInner {}
unsafe impl Send for TaskInner {}

#[derive(Clone)]
pub struct Task(Arc<TaskInner>);

impl Task {

    async fn with_sn(mgr: PingManager, remote: DeviceObject) -> NearResult<Self> {
        let task = Self(Arc::new(TaskInner {
            mgr: mgr.clone(),
            remote: remote.try_into()?,
            sessions: vec![],
        }));

        let mut sessions = vec![];
        // add tcp session
        for endpoint in task.as_remote_object().body().content().endpoints().iter().filter(| ep | ep.is_tcp() ) {
            match SessionInner::with_tcp(task.clone(), endpoint.clone()) {
                Ok(session) => sessions.push(Arc::new(session)),
                Err(e) => {
                    warn!("failed connect in session with err: {e}");
                }
            }
        }

        // add udp session
        let vports = mgr.vports().await;
        for endpoint in task.as_remote_object().body().content().endpoints().iter().filter(| ep | ep.is_udp() ) {
            for vport in vports.iter() {
                match SessionInner::with_udp(task.clone(), vport.clone(), endpoint.clone()) {
                    Ok(session) => sessions.push(Arc::new(session)),
                    Err(e) => {
                        warn!("failed connect in session with err: {e}");
                    }
                }
            }
        }

        std::mem::swap(&mut unsafe { &mut *(Arc::as_ptr(&task.0) as *mut TaskInner) }.sessions, &mut sessions);

        Ok(task)

    }

    async fn with_box(mgr: PingManager, remote: DeviceObject) -> NearResult<Self> {
        let task = Self(Arc::new(TaskInner {
            mgr: mgr.clone(),
            remote: remote.try_into()?,
            sessions: vec![],
        }));

        let mut sessions = vec![];

        // add udp session
        let vports = mgr.vports().await;
        for endpoint in task.as_remote_object().body().content().reverse_endpoint_array().iter().map(| pair | pair.remote() ).filter(| &ep | ep.is_udp() ) {
            for vport in vports.iter() {
                match SessionInner::with_udp(task.clone(), vport.clone(), endpoint.clone()) {
                    Ok(session) => sessions.push(Arc::new(session)),
                    Err(e) => {
                        warn!("failed connect in session with err: {e}");
                    }
                }
            }
        }

        std::mem::swap(&mut unsafe { &mut *(Arc::as_ptr(&task.0) as *mut TaskInner) }.sessions, &mut sessions);

        Ok(task)
    }

    pub(crate) async fn new(mgr: PingManager, remote: DeviceObject) -> NearResult<Self> {

        let codec = remote.object_id().object_type_code()?;
        let task = 
            match codec {
                ObjectTypeCode::Service(sub_codec) if sub_codec == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_SN_MINER as u8 => {
                    Task::with_sn(mgr, remote).await
                }
                ObjectTypeCode::Device(sub_codec) if  sub_codec == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => {
                    Task::with_box(mgr, remote).await
                }
                _ => {
                    Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("device object codec must service object or device object, expr: {}.", codec)))
                }
            }?;

        Ok(task)
    }

    pub(crate) async fn reset(&self, endpoint: Option<Endpoint>) -> NearResult<()> {
        let sessions: Vec<Arc<SessionInner>> = 
            if let Some(endpoint) = endpoint {
                self.0.sessions
                    .iter()
                    .filter(| session | {
                        session.remote_endpoint == endpoint
                    })
                    .cloned()
                    .collect()
            } else {
                self.0.sessions.clone()
            };

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

    pub(crate) async fn vport_array(&self) -> Vec<EndpointPair> {
        let mut array = vec![];

        for session in self.0.sessions.iter() {
            match &*session.remote_status.read().await {
                SessionInnerStatus::Established(status) if status.remote_external.is_some() => {
                    if let SessionNetwork::Udp(vport) = &session.session_type {
                        array.push(
                            EndpointPair::new(
                                vport.local_address().clone(), 
                                status.remote_external.as_ref().unwrap().clone()
                            )
                        );
                    } else {

                    }
                }
                _ => {}
            }
        }

        array
    }

    pub async fn call_peer(&self, remote: &ObjectId) -> NearResult<CallResultRef> {

        let random_index = now() as usize % self.0.sessions.len();

        self.0.sessions.get(random_index)
            .ok_or_else(|| {
                let error_string = format!("not found task session, index: {}, len: {}", random_index, self.0.sessions.len());
                error!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
            })?
            .call_peer(remote)
            .await
    }

    // pub(crate) async fn stop(&self) {
    //     let mut_fut = &mut *self.fut.write().await;

    //     match mut_fut {
    //         Some(mut_fut) => {
    //             self.inner.stop();
    //             let _ = mut_fut.await;
    //         }
    //         None => {
    //             // unactived
    //         }
    //     }

    //     *mut_fut = None;
    // }

}

#[async_trait::async_trait]
impl PackageEstablishedTrait for Task {
    async fn on_established(
        &self,
        tunnel: DynamicTunnel
    ) {
        self.0.on_established(tunnel).await
    }
}

impl Task {
    pub async fn on_ping_resp(
        &self,
        resp: PingResp,
    ) -> NearResult<()> {
        self.0.on_ping_resp(resp).await
    }
}

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
impl PostMessageTrait<(HeaderMeta, Data)> for Task {

    async fn post_message(&self, context: (HeaderMeta, Data)) -> NearResult<()> {

        self.get_session()
            .await?
            .post_message(context)
            .await

    }
}

impl Task {
    async fn get_session(&self) -> NearResult<Arc<SessionInner>> {
        let sessions = self.0.sessions.clone();

        let sessions_count = sessions.len();
        if sessions_count == 0 {
            warn!("no found sessions");
            return Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "no found sessions"));
        }

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
