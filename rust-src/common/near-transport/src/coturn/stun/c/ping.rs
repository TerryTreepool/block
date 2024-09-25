
use std::{
    collections::{btree_map::Entry, BTreeMap}, net::{IpAddr, Ipv4Addr, SocketAddr}, sync::{atomic::AtomicBool, Arc}
};

use async_std::sync::RwLock;

use log::{debug, error, info, trace, warn, };
use rand::{thread_rng, Rng};

use near_base::{
    device::DeviceId, sequence::SequenceString, DeviceObject, DeviceObjectSubCode, Endpoint, ErrorCode, NearError, NearResult, ObjectId, ObjectTypeCode, ServiceObjectSubCode, Signature
};
use crate::{
    network::{DataContext, Udp}, 
    package::*, 
    process::{PackageEstablishedTrait, PackageFailureTrait}, 
    coturn::stun::p::BaseEventManager, 
    tunnel::PostMessageTrait, 
};
use crate::{tunnel::DynamicTunnel, Stack};
use super::{task::Task, Config, NetworkAccessType};

#[derive(Default)]
struct TaskComponents {
    box_tasks: RwLock<BTreeMap<DeviceId, Arc<Task>>>,
    snm_tasks: RwLock<BTreeMap<DeviceId, Arc<Task>>>,
}

macro_rules! match_object_task {
    ($on:ident, $b1:tt, $b2:tt) => {
        match $on.object_type_code()? {
            ObjectTypeCode::Device(o) if o == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => $b1,
            ObjectTypeCode::Service(o) if o == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_COTURN_MINER as u8 => $b2,
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
                match self.box_tasks.write().await.entry(remote.clone()) {
                    Entry::Occupied(mut existed) => {
                        let des = Arc::as_ptr(existed.get_mut()) as *mut Task;
                        let src = Arc::as_ptr(&task) as *mut Task;
                        unsafe {
                            std::ptr::swap(des, src);
                        }
                    }
                    Entry::Vacant(empty) => {
                        let _ = empty.insert(task);
                    }
                }
        //         // let _ = self.box_tasks.write().await.entry(remote.clone()).or_insert(task);
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

struct NetworkAccessInfo {
    network_access_type: NetworkAccessType,
    network_mapping_address: Option<Endpoint>,
}

struct PingManagerImpl {
    stack: Stack,
    // config: Config,
    vports: RwLock<Vec<Udp>>,
    tasks: TaskComponents,
    actived: AtomicBool,
    fut: RwLock<Option<async_std::task::JoinHandle<()>>>,
    sender: async_std::channel::Sender<()>,
    recver: async_std::channel::Receiver<()>,

    network_access_info: RwLock<NetworkAccessInfo>,
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
    pub async fn init(stack: Stack) -> NearResult<Self> {
        let (sender, recver) = async_std::channel::bounded(8);

        let this = Self(Arc::new(PingManagerImpl {
            stack,
            vports: RwLock::new(vec![]),
            tasks: TaskComponents::default(),
            // tasks: RwLock::new(BTreeMap::new()),
            actived: AtomicBool::new(true),
            fut: RwLock::new(None),
            sender,
            recver,
            network_access_info: RwLock::new(NetworkAccessInfo {
                network_access_type: NetworkAccessType::NAT,
                network_mapping_address: None,
            }),
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
        &self.0.stack.config().peer_c_c
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

    pub(in super) async fn report_network_endpoint(&self, endpoint: &Endpoint) {
        let mut_network_info = &mut *self.0.network_access_info.write().await;

        match &mut_network_info.network_access_type {
            NetworkAccessType::NAT => {
                // Compare two external network mapping addresses
                // if they are the same, then the network is NAT
                // otherwise Symmetric
                if let Some(network_mapping_address) = mut_network_info.network_mapping_address.as_ref() {
                    if network_mapping_address != endpoint {
                        mut_network_info.network_access_type = NetworkAccessType::Symmetric;
                    }
                } else {
                    mut_network_info.network_mapping_address = Some(endpoint.clone());
                }
            }
            _ => {}
        }
    }

    pub(in super) async fn network_access_type(&self) -> NetworkAccessType {
        self.0.network_access_info.read().await.network_access_type
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
        for _ in 0..self.config().max_try_random_vport_times {
            let try_port = rng.gen_range(
                self.config().min_random_vport,
                self.config().max_random_vport,
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

            if let Err(_err) = async_std::future::timeout(self.config().ping_interval, async move {
                let _ = futures::future::join_all(mut_task_process).await;
            })
            .await {
                debug!("ping timeout.");
            } else {
                let recver = self.0.recver.clone();
                let _ = async_std::future::timeout(self.config().ping_interval, async move {
                    recver
                        .recv()
                        .await
                        .map_err(|e| NearError::new(ErrorCode::NEAR_ERROR_3RD, e.to_string()))
                })
                .await;

                debug!("ping finished.");
            }
        }

        Ok(())
    }
}

impl PingManager {
    async fn call_peer(
        &self, 
        sequence: &SequenceString,
        sn: Option<&ObjectId>,
        peer_id: &ObjectId
    ) -> NearResult<()> {
        let service_task = 
            self.0.tasks.get_service(sn)
                .await?
                .get(0)
                .cloned()
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "not found sn service")
                })?;

        info!("select {} to finished call peer proc, sequence: {sequence}", service_task.as_remote_object().object_id());

        if !service_task.wait_online(Some(self.config().ping_interval_connect)).await {
            let error_string = format!("{} service not actived.", service_task.as_remote_object().object_id());
            error!("{error_string}, sequence: {sequence}");
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNACTIVED, error_string))
        } else {
            Ok(())
        }?;

        service_task.call_peer(sequence, peer_id).await
            .map_err(| err | {
                error!("failed call-peer with err: {err} with called peer: {peer_id}, sequence: {sequence}");
                err
            })
    }

    async fn allocation_turn(
        &self,
        sequence: &SequenceString,
        sn: Option<&ObjectId>,
        peer_id: &ObjectId
    ) -> NearResult<()> {
        let service_task = 
            self.0.tasks.get_service(sn)
                .await?
                .get(0)
                .cloned()
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "not found sn service")
                })?;

        info!("select {} to finished allocation turn with peer: {peer_id} proc, sequence: {sequence}", service_task.as_remote_object().object_id());

        if !service_task.wait_online(Some(self.config().ping_interval_connect)).await {
            let error_string = format!("{} service not actived.", service_task.as_remote_object().object_id());
            error!("{error_string}, sequence: {sequence}");
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNACTIVED, error_string))
        } else {
            Ok(())
        }?;

        service_task.allocation_turn(sequence, peer_id).await
            .map_err(| err | {
                error!("failed allocation-turn with err: {err} with called peer: {peer_id}, sequence: {sequence}");
                err
            })

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

impl PingManager {
    pub(crate) async fn on_ping_request(
        &self,
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        ping: (StunReq, Signature),
    ) -> NearResult<StunReq> {

        let peer_id = head_ext.requestor();
        let sequence = head.sequence();
        let (ping, _) = ping;

        match ping.stun_type() {
            StunType::PingRequest => {},
            _ => debug_assert!(false, "not ping request")
        };

        trace!("{sequence} from: {peer_id}");

        Ok(StunReq::new(StunType::PingResponse))

    }

    pub(crate) async fn on_ping_response(
        &self,
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        data: (StunReq, Signature),
    ) -> NearResult<()> {

        let (resp, _) = data;
        let sequence = head.sequence();
        let requestor = head_ext.requestor();
        match resp.stun_type() {
            StunType::PingResponse | StunType::PingErrorResponse => {},
            _ => debug_assert!(false, "not ping response")
        };
        let log_key = format!(
            "[{} from {requestor} seq({sequence})]",
            resp.stun_name()
        );
        trace!("{}.", log_key);

        BaseEventManager::get_instance()
            .take_routine(
                head_ext.requestor(), 
                head.sequence(), 
                0,
            )
            .ok_or_else(|| {
                warn!("{log_key} not found event callback");
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found event callback")
            })?
            .emit(head, head_ext, resp.into())
            .await?;

        Ok(())
    }

    pub(crate) async fn on_call_request(
        &self,
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        data: (StunReq, Signature),
    ) -> NearResult<StunReq> {
        let (mut called, _signature) = data;
        let sequence = head.sequence();
        let package_source = head_ext.source();

        let fromer = 
            called.take_fromer().ok_or_else(|| {
                error!("not found former, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing former data")
            })?;
        let fromer_id = fromer.object_id();
        let peer_id = 
            called.take_target().ok_or_else(|| {
                error!("not found peer-id, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing peer id data")
            })?;

        let log_key = format!(
            "[{} from {fromer_id} seq({sequence})]",
            called.stun_name()
        );
        trace!("{}.", log_key);

        if &peer_id == self.as_stack().local_device_id() {
            Ok(())
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, "unmatch peer id, it not me."))
        }?;

        // add cache
        self.0.stack.cacher_manager().add(&fromer);

        // get vport
        let vport_array = {
            self.0.tasks.get_service(Some(package_source.requestor()))
                .await
                .map_err(| err | {
                    error!("[{log_key}] not found {} task with err: {}", package_source.requestor(), err);
                    err
                })?
                .get(0)
                .ok_or_else(|| {
                    error!("[{log_key}] not enough task.");
                    NearError::new(ErrorCode::NEAR_ERROR_NOT_ENOUGH, "not enough task")
                })?
                .vport_array()
                .await
        };

        let resp = 
            StunReq::new(StunType::CallResponse)
                .set_fromer(Some({
                    // 携带设备信息
                    let mut local = 
                        self.as_stack().cacher_manager().local();

                    let _ = 
                        std::mem::replace(
                            local.mut_body().mut_content().mut_reverse_endpoint_array(), 
                            vport_array
                        );

                    local
                }))
                .set_target(Some(peer_id));

        let this = self.clone();
        async_std::task::spawn(async move {
            info!("[{log_key}] box endpoint-pair: {:?}", fromer.body().content().reverse_endpoint_array());

            let _ = 
                this.add_sn(fromer).await
                    .map(| _ | {
                        info!("[{log_key}] add-sn ok");
                    })
                    .map_err(|err| {
                        error!("[{log_key}] failed add-sn with err: {err}");
                        err
                    });
        });

        Ok(resp)
    }

    pub(crate) async fn on_call_response(
        &self,
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        data: (StunReq, Signature),
    ) -> NearResult<()> {

        let (resp, _) = data;
        let requestor = head_ext.requestor();
        let sequence = head.sequence();

        let log_key = format!(
            "[{} from {requestor} seq({sequence})]",
            resp.stun_name()
        );
        trace!("{}.", log_key);

        match resp.stun_type() {
            StunType::CallResponse | StunType::CallErrorResponse => {},
            _ => {
                debug_assert!(false, "not call response");
            }
        }

        BaseEventManager::get_instance()
            .take_routine(
                head_ext.requestor(), 
                head.sequence(), 
                0
            )
            .ok_or_else(|| {
                warn!("{log_key} not found event callback",);
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found event callback")
            })?
            .emit(head, head_ext, resp.into())
            .await?;

        Ok(())

    }

    pub(crate) async fn on_allocation_request(
        &self,
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        data: (StunReq, Signature),
    ) -> NearResult<StunReq> {
        let (mut allocation, _signature) = data;
        let sequence = head.sequence();
        let _package_source = head_ext.source();

        let fromer = 
            head_ext.source().creator().ok_or_else(|| {
                error!("not found creator, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing creator data")
            })?;
        let target = 
            allocation.take_target().ok_or_else(|| {
                error!("not found target-id, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing target data")
            })?;

        let log_key = format!("[{} from {fromer} to {target} seq({sequence})]", allocation.stun_name());
        trace!("{}.", log_key);

        let mix_hash = 
            allocation.take_mixhash().ok_or_else(||{
                error!("not found mix-hash, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing mix-hash data")
            })?;
        let live_minutes = 
            allocation.take_live_minutes().ok_or_else(||{
                error!("not found live-minutes, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing live-minutes data")
            })?;

        let proxy_address = 
            allocation.take_proxy_address().ok_or_else(|| {
                error!("not found proxy-address, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing proxy-address data")
            })?;

        if &target == self.as_stack().local_device_id() {
            Ok(())
        } else {
            error!("unmatch peer id, it not me, sequence: {sequence}");
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, "unmatch peer id, it not me."))
        }?;

        self.0.stack.turn_task()
            .mix_hash_stubs()
            .append(target, mix_hash, live_minutes, proxy_address);

        // // get vport
        // let vport_array = {
        //     self.0.tasks.get_service(Some(package_source.requestor()))
        //         .await
        //         .map_err(| err | {
        //             error!("[{log_key}] not found {} task with err: {}", package_source.requestor(), err);
        //             err
        //         })?
        //         .get(0)
        //         .ok_or_else(|| {
        //             error!("[{log_key}] not enough task.");
        //             NearError::new(ErrorCode::NEAR_ERROR_NOT_ENOUGH, "not enough task")
        //         })?
        //         .vport_array()
        //         .await
        // };

        Ok(
            StunReq::new(StunType::AllocationChannelResponse)
                .set_target(Some(fromer.clone()))
        )
        // PostMessageTrait::post_message(
        //         &self.0.stack, 
        //         (
        //             Some(tunnel),
        //             RequestorMeta {
        //                 sequence: Some(sequence),
        //                 creator: Some(CreatorMeta {
        //                     creator: package_source.creator,
        //                     ..Default::default()
        //                 }),
        //                 need_sign: true,
        //                 ..Default::default()
        //             },
        //             AnyNamedRequest::with_calledresp(CalledResp {
        //                 result: ErrorCode::NEAR_ERROR_SUCCESS.into_u16() as u8,
        //                 info: Some({
        //                     // 携带设备信息
        //                     let mut local = 
        //                         self.as_stack().cacher_manager().local();

        //                     let _ = 
        //                         std::mem::replace(
        //                             local.mut_body().mut_content().mut_reverse_endpoint_array(), 
        //                             vport_array
        //                         );

        //                     local
        //                 }),
        //             }),
        //             None
        //         )
        //     )
        //     .await
        //     .map(| _ | {
        //         info!("[{}], successfully post (called-resp) message ", log_key);
        //     })
        //     .map_err(| err | {
        //         error!("[{}], failure post (called-resp) message with error: {}",
        //             log_key,
        //             err
        //         );
        //         err
        //     })?;

        // let (sequence, package) = 
        //     self.0.stack
        //         .build_package(BuildPackageV1 {
        //             target: Some(head_ext.to),
        //             sequence: Some(sequence),
        //             body: AnyNamedRequest::with_calledresp(CalledResp {
        //                 result: ErrorCode::NEAR_ERROR_SUCCESS.into_u16() as u8,
        //                 info: Some({
        //                     // 携带设备信息
        //                     let mut local = 
        //                         self.as_stack().cacher_manager().local();

        //                     let _ = 
        //                         std::mem::replace(
        //                             local.mut_body().mut_content().mut_reverse_endpoint_array(), 
        //                             vport_array
        //                         );

        //                     local
        //                 }),
        //             }),
        //             need_sign: true,
        //             ..Default::default()
        //         })
        //         .await
        //         .map_err(|err| {
        //             error!("[{log_key}] failed build CalledResp with err: {err}");
        //             err
        //         })?;

        // resp to sn
        // self.0.stack
        //     .tunnel_manager()
        //     .post_message((tunnel, sequence.clone(), package))
        //     .await
        //     .map_err(|err| {
        //         error!("[{log_key}] failed post message with err: {err}, sequence: {sequence}");
        //         err
        //     })?;

        // let this = self.clone();
        // async_std::task::spawn(async move {
        //     info!("[{log_key}] box endpoint-pair: {:?}", fromer.body().content().reverse_endpoint_array());

        //     let _ = 
        //         this.add_sn(fromer).await
        //             .map(| _ | {
        //                 info!("[{log_key}] add-sn ok");
        //             })
        //             .map_err(|err| {
        //                 error!("[{log_key}] failed add-sn with err: {err}");
        //                 err
        //             });
        // });

        // Ok(resp)
    }

    pub(crate) async fn on_allocation_response(
        &self,
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        data: (StunReq, Signature),
    ) -> NearResult<()> {

        let (resp, _) = data;
        let requestor = head_ext.requestor();
        let sequence = head.sequence();

        let log_key = format!(
            "[{} from {requestor} seq({sequence})]",
            resp.stun_name()
        );
        trace!("{}.", log_key);

        match resp.stun_type() {
            StunType::AllocationChannelResponse | StunType::AllocationChannelErrorResponse => {},
            _ => {
                debug_assert!(false, "not allocation response");
            }
        }

        BaseEventManager::get_instance()
            .take_routine(
                head_ext.requestor(), 
                head.sequence(), 
                0
            )
            .ok_or_else(|| {
                warn!("{log_key} not found event callback",);
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found event callback")
            })?
            .emit(head, head_ext, resp.into())
            .await?;

        Ok(())

    }

}

// #[async_trait::async_trait]
// impl PackageEventTrait<(Ping, Signature)> for PingManager {
//     async fn on_package_event(
//         &self,
//         tunnel: DynamicTunnel,
//         head: PackageHeader,
//         head_ext: PackageHeaderExt,
//         data: (Ping, Signature),
//     ) -> NearResult<()> {
//         let (ping, _) = data;

//         let peer_id = ping.peer_id;
//         let ping_sequence = ping.ping_sequence;
//         let (_, sequence) = head.split();
//         let (source, _, _) = head_ext.split();

//         trace!("peer_id: {peer_id}");

//         let package = 
//             PackageBuilder::build_head(sequence, None)
//                 .build_topic(
//                     None,
//                     self.0.stack.local_device_id().clone(),
//                     source.requestor,
//                     None,
//                 )
//                 .build_body(AnyNamedRequest::with_pingresp(
//                     StunReq::new(StunType::BindRequest)
//                 ))
//                 .build(Some(self.0.stack.as_signer()))
//                 .await
//                 .map_err(|err| {
//                     error!("failed build PingResp package to {} with {}", tunnel, err);
//                     err
//                 })?;

//         tunnel.post_message(package).await.map_err(|e| {
//             error!("failed send package with err: {}", e);
//             e
//         })?;

//         Ok(())
//     }
// }

// #[async_trait::async_trait]
// impl PackageEventTrait<(PingResp, Signature)> for PingManager {
//     async fn on_package_event(
//         &self,
//         tunnel: DynamicTunnel,
//         head: PackageHeader,
//         head_ext: PackageHeaderExt,
//         data: (PingResp, Signature),
//     ) -> NearResult<()> {

//         let (resp, _) = data;

//         log::info!(
//             "{} ping-resp, sn: {}, seq: {}.",
//             self,
//             resp.peer_id.to_string(),
//             resp.ping_sequence
//         );

//         BaseEventManager::get_instance()
//             .take_routine(
//                 head_ext.requestor(), 
//                 head.sequence(), 
//                 0,
//             )
//             .ok_or_else(|| {
//                 warn!("not found event callback, sequence: {}", head.sequence());
//                 NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found event callback")
//             })?
//             .emit(tunnel, head, head_ext, resp.into())
//             .await?;

//         Ok(())
//     }
// }

// impl PingManager {
//     pub async fn on_ping_resp(&self, resp: PingResp) -> NearResult<()> {
//         log::info!(
//             "{} ping-resp, sn: {}, seq: {}.",
//             self,
//             resp.peer_id.to_string(),
//             resp.ping_sequence
//         );

//         let task = 
//             self.0.tasks.get(&resp.peer_id).await
//                 .map_err(| err | {
//                     log::warn!(
//                         "{} ping-resp, sn: {} not found, maybe is stopped, err is {}.",
//                         self,
//                         resp.peer_id.to_string(),
//                         err
//                     );
//                     err
//                 })?;

//         let _ = task.on_ping_resp(resp).await;

//         Ok(())
//     }
// }

// #[async_trait::async_trait]
// impl PackageEventTrait<(CalledReq, Signature)> for PingManager {
//     async fn on_package_event(
//         &self,
//         tunnel: DynamicTunnel,
//         head: PackageHeader,
//         head_ext: PackageHeaderExt,
//         data: (CalledReq, Signature),
//     ) -> NearResult<()> {
//         let (called, _signature) = data;
//         let (_, sequence) = head.split();
//         let (package_source, _, _) = head_ext.split();
//         let log_key = format!(
//             "[called from {} seq({})]",
//             called.peer_info.object_id(),
//             sequence,
//         );
//         info!("{}.", log_key);

//         // add cache
//         self.0.stack.cacher_manager().add(&called.peer_info);

//         // get vport
//         let vport_array = {
//             self.0.tasks.get_service(Some(package_source.requestor()))
//                 .await
//                 .map_err(| err | {
//                     error!("[{log_key}] not found {} task with err: {}", package_source.requestor(), err);
//                     err
//                 })?
//                 .get(0)
//                 .ok_or_else(|| {
//                     error!("[{log_key}] not enough task.");
//                     NearError::new(ErrorCode::NEAR_ERROR_NOT_ENOUGH, "not enough task")
//                 })?
//                 .vport_array()
//                 .await
//         };

//         PostMessageTrait::post_message(
//                 &self.0.stack, 
//                 (
//                     Some(tunnel),
//                     RequestorMeta {
//                         sequence: Some(sequence),
//                         creator: Some(CreatorMeta {
//                             creator: package_source.creator,
//                             ..Default::default()
//                         }),
//                         need_sign: true,
//                         ..Default::default()
//                     },
//                     AnyNamedRequest::with_calledresp(CalledResp {
//                         result: ErrorCode::NEAR_ERROR_SUCCESS.into_u16() as u8,
//                         info: Some({
//                             // 携带设备信息
//                             let mut local = 
//                                 self.as_stack().cacher_manager().local();

//                             let _ = 
//                                 std::mem::replace(
//                                     local.mut_body().mut_content().mut_reverse_endpoint_array(), 
//                                     vport_array
//                                 );

//                             local
//                         }),
//                     }),
//                     None
//                 )
//             )
//             .await
//             .map(| _ | {
//                 info!("[{}], successfully post (called-resp) message ", log_key);
//             })
//             .map_err(| err | {
//                 error!("[{}], failure post (called-resp) message with error: {}",
//                     log_key,
//                     err
//                 );
//                 err
//             })?;

//         // let (sequence, package) = 
//         //     self.0.stack
//         //         .build_package(BuildPackageV1 {
//         //             target: Some(head_ext.to),
//         //             sequence: Some(sequence),
//         //             body: AnyNamedRequest::with_calledresp(CalledResp {
//         //                 result: ErrorCode::NEAR_ERROR_SUCCESS.into_u16() as u8,
//         //                 info: Some({
//         //                     // 携带设备信息
//         //                     let mut local = 
//         //                         self.as_stack().cacher_manager().local();

//         //                     let _ = 
//         //                         std::mem::replace(
//         //                             local.mut_body().mut_content().mut_reverse_endpoint_array(), 
//         //                             vport_array
//         //                         );

//         //                     local
//         //                 }),
//         //             }),
//         //             need_sign: true,
//         //             ..Default::default()
//         //         })
//         //         .await
//         //         .map_err(|err| {
//         //             error!("[{log_key}] failed build CalledResp with err: {err}");
//         //             err
//         //         })?;

//         // resp to sn
//         // self.0.stack
//         //     .tunnel_manager()
//         //     .post_message((tunnel, sequence.clone(), package))
//         //     .await
//         //     .map_err(|err| {
//         //         error!("[{log_key}] failed post message with err: {err}, sequence: {sequence}");
//         //         err
//         //     })?;

//         let this = self.clone();
//         async_std::task::spawn(async move {
//             info!("[{log_key}] box endpoint-pair: {:?}", called.peer_info.body().content().reverse_endpoint_array());

//             let _ = 
//                 this.add_sn(called.peer_info).await
//                     .map(| _ | {
//                         info!("[{log_key}] add-sn ok");
//                     })
//                     .map_err(|err| {
//                         error!("[{log_key}] failed add-sn with err: {err}");
//                         err
//                     });
//         });

//         Ok(())
//     }
// }

// #[async_trait::async_trait]
// impl PackageEventTrait<(CallResp, Signature)> for PingManager {
//     async fn on_package_event(
//         &self,
//         tunnel: DynamicTunnel,
//         head: PackageHeader,
//         head_ext: PackageHeaderExt,
//         data: (CallResp, Signature),
//     ) -> NearResult<()> {

//         let (resp, _) = data;

//         BaseEventManager::get_instance()
//             .take_routine(
//                 head_ext.requestor(), 
//                 head.sequence(), 
//                 0
//             )
//             .ok_or_else(|| {
//                 warn!("not found event callback, sequence: {}", head.sequence());
//                 NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found event callback")
//             })?
//             .emit(tunnel, head, head_ext, resp.into())
//             .await?;

//         Ok(())

//     }
// }

#[async_trait::async_trait]
impl PackageFailureTrait for PingManager {

    async fn on_package_failure(
        &self, 
        error: NearError,
        data: DataContext,
    ) {
        let sequence = data.head.sequence();
        let target = data.head_ext.to();
        let major_command = data.head.major_command();

        trace!("PingManager::on_package_failure, sequence: {sequence}, major_command: {major_command}, target: {target}, error: {error}");

        if let Some(event) = BaseEventManager::get_instance().take_routine(target, sequence, 0) {
            event.emit_error(error, data).await;
        } else {
            warn!("not found event callback, sequence: {sequence}");
        }
    }

}


#[async_trait::async_trait]
impl PostMessageTrait<(ObjectId, SequenceString, PackageDataSet)> for PingManager {

    type R = ();

    async fn post_message(
        &self, 
        context: (ObjectId, SequenceString, PackageDataSet)
    ) -> NearResult<Self::R> {

        let (target, sequence, package) = context;

        trace!("PingManager::post_message: target: {}, sequence: {},", target, sequence);

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
            match self.0.tasks.get_device(&target).await {
                Ok(task) => {
                    if task.is_active() {
                        Ok(Some(task))
                    } else {
                        info!("task was dead sequence: {sequence}, will re-create it's tunnel");
                        Ok(None)
                    }
                }
                Err(err) => {
                    match err.errno() {
                        ErrorCode::NEAR_ERROR_NOTFOUND => {
                            info!("not found {target} sequence: {sequence}, will create it's tunnel");
                            Ok(None)
                        }
                        _ => Err(err)
                    }
                }
            }?;

        let task = 
            if let Some(task) = task {
                task
            } else {
                self.call_peer(&sequence, None, &target).await?;

                self.0.tasks.get_device(&target).await?
            };

        // Will post or turn message, by the task's net work access type.
        let network_access_type = task.network_access_type().await;
        let network_access_type = 
            if network_access_type == NetworkAccessType::Unknown {
                self.network_access_type().await
            } else {
                network_access_type
            };

        enum NextStep {
            Over,
            Turn,
            Error(NearError),
        }
        let next_step = 
            if network_access_type == NetworkAccessType::NAT {
                if let Err(err) = task.post_message((target.clone(), sequence.clone(), package.clone())).await {
                    match err.errno() {
                        ErrorCode::NEAR_ERROR_UNACTIVED => {

                            // remove sn in ping manager, because it's network Symmetric type 
                            self.remove_sn(&target).await?;

                            // wait online timeout, the task's network access type will change NetworkAccessType::Symmetric
                            task.change_network_access_type(NetworkAccessType::Symmetric).await;

                            // request stun-allocation to stun-server
                            self.allocation_turn(&sequence, None, &target)
                                .await
                                .map_err(| err | {
                                    let error_string = format!("post stun-allocation message with err: {err}");
                                    error!("{error_string}, sequence: {sequence}");
                                    err
                                })?;

                            NextStep::Turn
                        }
                        _ => {
                            NextStep::Error(err)
                        }
                    }
                } else {
                    // check transfer key's validation with peer
                    if !self.0.stack.turn_task().mix_hash_stubs().is_valid(&target) {
                        // request stun-allocation to stun-server
                        self.allocation_turn(&sequence, None, &target)
                            .await
                            .map_err(| err | {
                                let error_string = format!("post stun-allocation message with err: {err}");
                                error!("{error_string}, sequence: {sequence}");
                                err
                            })?;
                    }

                    NextStep::Over
                }
            } else {

                NextStep::Turn
            };

        match next_step {
            NextStep::Turn => {
                if let Err(err) = self.0.stack.turn_task().post_message((target, sequence, package)).await {
                    let error_string = format!("post message with err: {err}");
                    error!("{error_string}, sequence: {sequence}");
                    Err(err)
                } else {
                    Ok(())
                }
            }
            NextStep::Over => Ok(()),
            NextStep::Error(err) => Err(err)
        }
    }
}
