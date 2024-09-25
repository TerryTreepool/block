use async_std::sync::RwLock;
use std::{
    collections::BTreeMap, net::{IpAddr, Ipv4Addr, SocketAddr}, pin::Pin, sync::{atomic::AtomicBool, Arc}
};

use log::{debug, error, info, trace, };
use near_base::{
    device::DeviceId, DeviceObject, DeviceObjectSubCode, Endpoint, ErrorCode, NearError, NearResult, ObjectId, ObjectTypeCode, ServiceObjectSubCode, Signature
};
use rand::{thread_rng, Rng};

use crate::{
    h::OnBuildPackage, network::Udp, package::*, process::PackageEstablishedTrait, sn::c::call::{CallCenterManager, CallSessionStatus}, stack::BuildPackageV1, tunnel::PostMessageTrait, HeaderMeta, PackageEventTrait
};
use crate::{tunnel::DynamicTunnel, Stack};

use super::{call::CallSession, task::Task, Config};

#[derive(Default)]
struct TaskComponents {
    box_tasks: RwLock<BTreeMap<DeviceId, Arc<Task>>>,
    snm_tasks: RwLock<BTreeMap<DeviceId, Arc<Task>>>,
}

macro_rules! match_object_task {
    ($on:ident, $b1:tt, $b2:tt) => {
        match $on.object_type_code()? {
            ObjectTypeCode::Device(o) if o == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => $b1,
            ObjectTypeCode::Service(o) if o == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_SN_MINER as u8 => $b2,
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "invalid device object"))
        }
    };

}

impl TaskComponents {
    async fn get(&self, remote: &ObjectId) -> NearResult<Arc<Task>> {
        match_object_task!(
            remote,
            {
                self.box_tasks
                    .read().await
                    .get(remote)
                    .cloned()
                    .ok_or_else(|| {
                        NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{remote} not found"))
                    })
            },
            {
                self.snm_tasks
                    .read().await
                    .get(remote)
                    .cloned()
                    .ok_or_else(|| {
                        NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{remote} not found"))
                    })
            }
        )
    }

    #[allow(unused)]
    async fn get_device(&self, remote: &ObjectId) -> NearResult<Arc<Task>> {
        match_object_task!(
            remote,
            {
                self.box_tasks
                    .read().await
                    .get(remote)
                    .cloned()
                    .ok_or_else(|| {
                        NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{remote} not found"))
                    })
            },
            {
                Err(NearError::new(ErrorCode::NEAR_ERROR_INCORRECT_USE, format!("{remote} isn't device.")))
            }
        )
    }

    #[allow(unused)]
    async fn get_service(&self, remote: Option<&ObjectId>) -> NearResult<Vec<Arc<Task>>> {
        if let Some(remote) = remote {
            Ok(
                [
                    self.snm_tasks.read().await.get(remote)
                        .cloned()
                        .ok_or_else(|| {
                            NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{remote} not found"))
                        })?
                ]
                .to_vec()
            )
        } else {
            Ok(
                self.snm_tasks.read().await
                    .values()
                    .map(| task | task.clone())
                    .collect()
            )
        }
    }

    async fn all(&self) -> Vec<Arc<Task>> {
        let r1: Vec<Arc<Task>> = 
            self.box_tasks.read().await
                .values()
                .map(|task| task.clone())
                .collect();

        let r2: Vec<Arc<Task>> = 
            self.snm_tasks.read().await
                .values()
                .map(|task| task.clone())
                .collect();

        [r1, r2].concat()
    }

    async fn add(&self, remote: &ObjectId, task: Arc<Task>) -> NearResult<()> {
        match_object_task!(
            remote, 
            {
                let _ = self.box_tasks.write().await.entry(remote.clone()).or_insert(task);
                Ok(())
            },
            {
                let _ = self.snm_tasks.write().await.entry(remote.clone()).or_insert(task);
                Ok(())
            }
        )
    }

    async fn remove(&self, remote: &ObjectId) -> NearResult<Arc<Task>> {
        match_object_task!(
            remote,
            {
                self.box_tasks.write().await.remove(remote).ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{remote} not found"))
                })
            },
            {
                self.snm_tasks.write().await.remove(remote).ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{remote} not found"))
                })
            }
        )
    }
}

struct PingManagerImpl {
    stack: Stack,
    config: Config,
    vports: RwLock<Vec<Udp>>,
    tasks: TaskComponents,
    // tasks: RwLock<BTreeMap<DeviceId, Arc<Task>>>,
    actived: AtomicBool,
    fut: RwLock<Option<async_std::task::JoinHandle<()>>>,
    sender: async_std::channel::Sender<()>,
    recver: async_std::channel::Receiver<()>,
}

impl std::ops::Drop for PingManagerImpl {
    fn drop(&mut self) {
        let this = self;

        async_std::task::block_on(async move {
            this.actived.store(false, std::sync::atomic::Ordering::SeqCst);

            let mut_fut = &mut *this.fut.write().await;
            if let Some(handle) = mut_fut.as_mut() {
                handle.await;
                *mut_fut = None;
            }
        });
    }
}

#[derive(Clone)]
pub(crate) struct PingManager(Arc<PingManagerImpl>);

impl PingManager {
    pub async fn init(stack: Stack, config: Option<Config>) -> NearResult<Self> {
        let (sender, recver) = async_std::channel::bounded(8);

        let this = Self(Arc::new(PingManagerImpl {
            stack,
            config: config.unwrap_or_default(),
            vports: RwLock::new(vec![]),
            tasks: TaskComponents::default(),
            // tasks: RwLock::new(BTreeMap::new()),
            actived: AtomicBool::new(true),
            fut: RwLock::new(None),
            sender,
            recver,
        }));

        let fut = this.start().await?;

        *this.0.fut.write().await = Some(fut);

        Ok(this)
    }

    #[inline]
    pub(crate) async fn vports(&self) -> Vec<Udp> {
        self.0.vports.read().await.iter().cloned().collect()
    }

    #[inline]
    pub(crate) fn as_stack(&self) -> &Stack {
        &self.0.stack
    }

    #[inline]
    pub(crate) fn config(&self) -> &Config {
        &self.0.config
    }

    pub async fn reset_sn(
        &self,
        remote: DeviceObject,
        endpoint: Option<Endpoint>,
    ) -> NearResult<()> {
        let remote_id = remote.object_id();

        let task = 
            self.0.tasks.get(remote_id)
                .await
                .map_err(| err | {
                    error!("failed reset task for {err}");
                    err
                })?;
        // let task = { self.0.tasks.read().await.get(remote_id).cloned() }.ok_or_else(|| {
        //     let error_string = format!("Not found {remote_id} target-sn.");
        //     error!("{error_string}");
        //     NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
        // })?;

        task.reset(endpoint).await
    }

    pub async fn add_sn(&self, remote: DeviceObject) -> NearResult<()> {
        let remote_id = remote.object_id().clone();

        let new_task = Arc::new(Task::new(self.clone(), remote).await?);

        self.as_stack().cacher_manager().add(new_task.as_remote_object());

        self.0.tasks.add(&remote_id, new_task).await?;

        info!("successfully add {} into ping-manager.", remote_id);

        let _ = 
            self.0.sender.send(())
                .await
                .map_err(|e| {
                    error!("remove_sn, ping_trigger.send err: {}", e);
                });

        Ok(())
    }

    pub async fn remove_sn(&self, remote_id: &DeviceId) -> NearResult<()> {
        let task = self.0.tasks.remove(remote_id).await?;

        let _ = 
            self.0.sender.send(()).await.map_err(|e| {
                error!("remove_sn, ping_trigger.send err: {}", e);
            });

        info!("task: {} will remove.", task);

        Ok(())

    }

    pub(self) async fn start(&self) -> NearResult<async_std::task::JoinHandle<()>> {
        self.init_vport().await?;

        let this = self.clone();

        let fut = async_std::task::spawn(async move {
            let _ = this.on_process().await;
        });

        Ok(fut)
    }

    pub(self) async fn init_vport(&self) -> NearResult<()> {
        async fn bind_udp_socket(stack: &Stack, port: u16) -> NearResult<Udp> {
            stack
                .net_manager()
                .bind_udp_interface(&Endpoint::default_udp(SocketAddr::new(
                    IpAddr::from(Ipv4Addr::new(0, 0, 0, 0)),
                    port,
                )))
                .await
        }

        let mut vports = vec![];
        let mut rng = thread_rng();
        for _ in 0..self.0.config.max_try_random_vport_times {
            let try_port = rng.gen_range(
                self.0.config.min_random_vport,
                self.0.config.max_random_vport,
            );

            if let Ok(interface) = bind_udp_socket(self.as_stack(), try_port).await {
                vports.push(interface);
            }
        }

        if vports.len() == 0 {
            Err(NearError::new(
                ErrorCode::NEAR_ERROR_UNINITIALIZED,
                "failed to bind vport.",
            ))
        } else {
            let mut_vports = &mut *self.0.vports.write().await;
            let _ = std::mem::replace(mut_vports, vports);
            Ok(())
        }
    }

    pub(self) async fn on_process(&self) -> NearResult<()> {
        loop {
            if !self.0.actived.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            let tasks = { self.0.tasks.all().await };

            let mut mut_task_process = vec![];

            for task in tasks.iter() {
                mut_task_process.push(task.on_process());
            }

            let _ = futures::future::join_all(mut_task_process).await;

            debug!("ping finished.");

            let recver = self.0.recver.clone();
            let _ = async_std::future::timeout(self.0.config.ping_interval, async move {
                recver
                    .recv()
                    .await
                    .map_err(|e| NearError::new(ErrorCode::NEAR_ERROR_3RD, e.to_string()))
            })
            .await;

        }

        Ok(())
    }
}

impl PingManager {
    async fn call_peer(
        &self, 
        sn: Option<&ObjectId>,
        peer_id: &ObjectId
    ) -> NearResult<Pin<Box<dyn std::future::Future<Output=CallSession> + Send + Sync>>> {
        let service_task = 
            self.0.tasks.get_service(sn)
                .await?
                .get(0)
                .cloned()
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "not found sn service")
                })?;

        info!("select {} to finished call peer proc", service_task.as_remote_object().object_id());

        let r = 
            service_task.call_peer(peer_id).await
                .map_err(| err | {
                    let error_string = format!("failed call-peer with err: {err} with called peer: {peer_id}");
                    error!("{error_string}");
                    err
                })
                .map(| f | Box::pin(f))?;

        Ok(r)
    }
}

impl std::fmt::Display for PingManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PingManager{{local:{}}}", self.0.stack.local_device_id())
    }
}

#[async_trait::async_trait]
impl PackageEstablishedTrait for PingManager {
    async fn on_established(&self, tunnel: DynamicTunnel) {
        if let Ok(task) = self.0.tasks.get(tunnel.peer_id()).await {
            task.on_established(tunnel).await
        }
    }
}

#[async_trait::async_trait]
impl PackageEventTrait<(Ping, Signature)> for PingManager {
    async fn on_package_event(
        &self,
        tunnel: DynamicTunnel,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        data: (Ping, Signature),
    ) -> NearResult<()> {
        let (ping, _) = data;

        let peer_id = ping.peer_id;
        let ping_sequence = ping.ping_sequence;
        let (_, sequence) = head.split();
        let (source, _, _) = head_ext.split();

        trace!("peer_id: {peer_id}");

        let package = 
            PackageBuilder::build_head(sequence, None)
                .build_topic(
                    None,
                    self.0.stack.local_device_id().clone(),
                    source.requestor,
                    None,
                )
                .build_body(AnyNamedRequest::with_pingresp(PingResp {
                        session_id: ping.session_id,
                        ping_sequence: ping_sequence,
                        peer_id: self.0.stack.local_device_id().clone(),
                        reverse_endpoint: None,
                    })
                )
                .build(Some(self.0.stack.as_signer()))
                .await
                .map_err(|err| {
                    error!("failed build PingResp package to {} with {}", tunnel, err);
                    err
                })?;

        tunnel.post_message(package).await.map_err(|e| {
            error!("failed send package with err: {}", e);
            e
        })?;

        Ok(())
    }
}

impl PingManager {
    pub async fn on_ping_resp(&self, resp: PingResp) -> NearResult<()> {
        log::info!(
            "{} ping-resp, sn: {}, seq: {}.",
            self,
            resp.peer_id.to_string(),
            resp.ping_sequence
        );

        let task = 
            self.0.tasks.get(&resp.peer_id).await
                .map_err(| err | {
                    log::warn!(
                        "{} ping-resp, sn: {} not found, maybe is stopped, err is {}.",
                        self,
                        resp.peer_id.to_string(),
                        err
                    );
                    err
                })?;

        let _ = task.on_ping_resp(resp).await;

        Ok(())
    }
}

#[async_trait::async_trait]
impl PackageEventTrait<(CalledReq, Signature)> for PingManager {
    async fn on_package_event(
        &self,
        tunnel: DynamicTunnel,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        data: (CalledReq, Signature),
    ) -> NearResult<()> {
        let (called, _signature) = data;
        let (_, sequence) = head.split();
        let log_key = format!(
            "[called from {} seq({})]",
            called.peer_info.object_id(),
            sequence,
        );
        info!("{}.", log_key);

        // add cache
        self.0.stack.cacher_manager().add(&called.peer_info);

        let (sequence, package) = 
            self.0.stack
                .build_package(BuildPackageV1 {
                    remote: Some(head_ext.to),
                    sequence: Some(sequence),
                    body: AnyNamedRequest::with_calledresp(CalledResp {
                        result: ErrorCode::NEAR_ERROR_SUCCESS.into_u16() as u8,
                    }),
                    need_sign: true,
                    ..Default::default()
                })
                .await
                .map_err(|err| {
                    error!("[{log_key}] failed build CalledResp with err: {err}");
                    err
                })?;

        // resp to sn
        self.0.stack
            .tunnel_manager()
            .post_message((tunnel, sequence.clone(), package))
            .await
            .map_err(|err| {
                error!("[{log_key}] failed post message with err: {err}, sequence: {sequence}");
                err
            })?;

        let this = self.clone();
        async_std::task::spawn(async move {
            let _ = 
                this.add_sn(called.peer_info).await
                    .map(| _ | {
                        info!("[{log_key}] add-sn ok");
                    })
                    .map_err(|err| {
                        error!("[{log_key}] failed add-sn with err: {err}");
                        err
                    });
        });

        Ok(())
    }
}

#[async_trait::async_trait]
impl PackageEventTrait<(CallResp, Signature)> for PingManager {
    async fn on_package_event(
        &self,
        _tunnel: DynamicTunnel,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        data: (CallResp, Signature),
    ) -> NearResult<()> {

        let (source, _target, _) = head_ext.split();
        let (resp, _) = data;
        let call_sequence = resp.call_sequence;
        let to_peer_info = resp.to_peer_info.ok_or_else(|| {
            NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing peer desc data.")
        })?;
        let to_peer_id = to_peer_info.object_id().clone();
        let errno: ErrorCode = (resp.result as u16).into();
        log::trace!(
            "sequence: {} call-resp, result: {}, peer-id: {:?}, call-seq: {}.",
            head.sequence(),
            errno,
            to_peer_id,
            call_sequence
        );

        match errno {
            ErrorCode::NEAR_ERROR_SUCCESS => {
                CallCenterManager::get_instance()
                    .call_result(source.requestor, to_peer_id, call_sequence, CallSessionStatus::Established)
                    .await;

                let _ = self.add_sn(to_peer_info).await?;
            }
            _ => {
                CallCenterManager::get_instance()
                    .call_result(source.requestor, to_peer_id, call_sequence, CallSessionStatus::Closed(errno))
                    .await;
            }
        };

        Ok(())

    }
}

#[async_trait::async_trait]
impl PostMessageTrait<(HeaderMeta, Data)> for PingManager {

    async fn post_message(&self, context: (HeaderMeta, Data)) -> NearResult<()> {

        let (header_meta, data) = context;

        trace!("PingManager::post_message: header_meta: {},", header_meta);

        let target = &header_meta.to;
        let target_type_codec = target.object_type_code()?;

        match target_type_codec {
            ObjectTypeCode::Device(v) if v == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => {
                Ok(())
            }
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("Unmatch device, expr: {}", target_type_codec)))
            }
        }?;

        let task = 
            match self.0.tasks.get_device(target).await {
                Ok(task) => Ok(task),
                Err(err) => {
                    match err.errno() {
                        ErrorCode::NEAR_ERROR_NOTFOUND => {
                            info!("not found {target}, will create it's tunnel");
                            match self.call_peer(None, target).await?.await.status() {
                                CallSessionStatus::Established => self.0.tasks.get_device(target).await,
                                CallSessionStatus::Closed(err) => {
                                    error!("failed call_peer({target}) with err: {err}");
                                    Err(NearError::new(*err, format!("failed call_peer({target})")))
                                }
                                CallSessionStatus::Connecting => unreachable!("don't reach here.")
                            }
                        }
                        _ => Err(err)
                    }
                }
            }?;

        task.post_message((header_meta, data)).await
    }
}
