use std::{
    collections::BTreeMap, 
    time::Duration, 
    sync::{atomic::{AtomicU64, Ordering}, Arc}
};

use async_std::sync::Mutex;
use log::{error, info, trace, warn};

use near_base::{device::DeviceId, *};

use crate::h::{OnBuildPackage, OnTimeTrait};
use crate::package::*;
use crate::stack::BuildPackageV1;
use crate::tunnel::{DynamicTunnel, PostMessageTrait};
use crate::PackageEventTrait;
use crate::Stack;

pub struct FoundPeer {
    pub desc: DeviceObject,
}

struct CachedPeerInfo {
    last_signature: Signature,
    last_send_time: Timestamp,
    last_call_time: Timestamp,
    last_checkout_time: Timestamp,
    last_ping_sequence: u32,
}

impl CachedPeerInfo {
    fn new(
        last_signature: Signature,
        send_time: Timestamp, 
        ping_sequence: u32, 
    ) -> CachedPeerInfo {
        CachedPeerInfo {
            last_ping_sequence: ping_sequence,
            last_signature,
            last_send_time: send_time,
            last_call_time: 0,
            last_checkout_time: 0,
        }
    }

    // fn to_found_peer(&self) -> FoundPeer {
    //     FoundPeer {
    //         desc: self.desc.clone(), 
    //     }
    // }

    // fn update_key(&mut self, key: AesKey) {
    //     if let Some(k) = self.last_key.as_mut() {
    //         *k = key;
    //     } else {
    //         self.last_key = Some(key);
    //     }
    // }


    // fn update_desc(&mut self, desc: &DeviceObject, new_signature: &Signature) -> NearResult<()> {

    //     match new_signature.sign_time().cmp(&self.last_signature.sign_time()) {
    //         std::cmp::Ordering::Equal => Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, "signature time equal, ignore")),
    //         std::cmp::Ordering::Less => Err(NearError::new(ErrorCode::NEAR_ERROR_EXPIRED, "signature time expire")),
    //         std::cmp::Ordering::Greater => Ok(())
    //     }?;

    //     let _ = std::mem::replace(&mut self.desc, desc.clone());
    //     let _ = std::mem::replace(&mut self.last_signature, new_signature.clone());
        
    //     Ok(())
    // }
}

struct Peers {
    actived_peers: BTreeMap<DeviceId, CachedPeerInfo>,
    knocked_peers: BTreeMap<DeviceId, CachedPeerInfo>,
}

impl Peers {
    fn find_peer(&mut self, peerid: &DeviceId, reason: FindPeerReason) -> Option<&mut CachedPeerInfo> {
        let found_peer_cache = match self.actived_peers.get_mut(peerid) {
            Some(peer_cache) => {
                Some(peer_cache)
            },
            None => match self.knocked_peers.get_mut(peerid) {
                Some(peer_cache) => Some(peer_cache),
                None => None
            }
        };
    
        if let Some(found_peer_cache) = found_peer_cache {
            match reason {
                FindPeerReason::CallFrom(t) => {
                    if t > found_peer_cache.last_call_time {
                        found_peer_cache.last_call_time = t;
                    }
                    Some(found_peer_cache)
                },
                // FindPeerReason::Checkout(t) => {
                //     if t > found_peer_cache.last_checkout_time {
                //         found_peer_cache.last_checkout_time = t;
                //     }
                //     Some(found_peer_cache)
                // }
                FindPeerReason::Other => {
                    Some(found_peer_cache)
                }
            }
        } else {
            None
        }
    }
}


struct PeerManagerImpl {
    stack: Stack,
    peers: Mutex<Peers>, 
    last_knock_time: AtomicU64,
}

#[derive(Clone)]
pub struct PeerManager(Arc<PeerManagerImpl>);

pub enum FindPeerReason {
    // Checkout(Timestamp),
    CallFrom(Timestamp),
    Other,
}


impl PeerManager {
    pub fn new(stack: Stack) -> PeerManager {

        let this = Self(Arc::new(PeerManagerImpl {
            stack,
            peers: Mutex::new(Peers {
                actived_peers: Default::default(),
                knocked_peers: Default::default(),
            }),
            last_knock_time: AtomicU64::new(now()),
        }));

        {
            trace!("on_time_escape");
            let this = this.clone();
            let polling_interval = this.0.stack.config().peer_c_s.polling_interval;

            async_std::task::spawn(async move {
                loop {
                    let now = now();

                    this.on_time_escape(now);

                    let _ = async_std::future::timeout(polling_interval, async_std::future::pending::<()>()).await;
                }
            });
        }

        this
    }

    pub(in self) async fn peer_heartbeat(
        &self, 
        ping: Ping,
        peer_signature: Signature,
        endpoint_pair: EndpointPair,
    ) -> NearResult<PingResp> {

        let peer_id = ping.peer_id;
        let peer_desc = ping.peer_info;
        let send_time = ping.send_time;
        let ping_sequence = ping.ping_sequence;

        trace!("peer_id: {peer_id}");

        let peer_id = 
            if let Some(peer_desc) = peer_desc.as_ref() {
                if peer_desc.object_id() == peer_id {
                    Ok(peer_id)
                } else {
                    log::error!("peer-id missmatch");
                    Err(NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "peer-id missmatch"))
                }
            } else {
                Ok(peer_id)
            }?;

        let exist_cache_found = | cached_peer: &mut CachedPeerInfo, new_signature: Signature | -> NearResult<()> {
            if cached_peer.last_send_time > send_time {
                log::warn!("ping send-time little.");
                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "ping send-time little."))
            } else {
                Ok(())
            }?;

            match new_signature.sign_time().cmp(&cached_peer.last_signature.sign_time()) {
                std::cmp::Ordering::Equal => Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, "signature time equal, ignore")),
                std::cmp::Ordering::Less => Err(NearError::new(ErrorCode::NEAR_ERROR_EXPIRED, "signature time expire")),
                std::cmp::Ordering::Greater => Ok(())
            }?;

            let _ = std::mem::replace(&mut cached_peer.last_signature, new_signature);

            cached_peer.last_send_time = send_time;
            cached_peer.last_ping_sequence = ping_sequence;

            Ok(())
        };

        let peers = &mut *self.0.peers.lock().await;

        // 1.从活跃peer中搜索已有cache
        if let Some(p) = peers.actived_peers.get_mut(&peer_id) {
            exist_cache_found(p, peer_signature)?;
            if let Some(peer_desc) = peer_desc {
                self.0.stack.cacher_manager().add(&peer_desc);
            }
            Ok(())
        } else {
            // 2.从待淘汰peer中搜索已有cache
            match peers.knocked_peers.remove(&peer_id) {
                Some(mut cached_found) => {
                    exist_cache_found(&mut cached_found, peer_signature)?;
                    if let Some(peer_desc) = peer_desc {
                        self.0.stack.cacher_manager().add(&peer_desc);
                    }
                    let _ = peers.actived_peers.insert(peer_id, cached_found);

                    Ok(())
                }
                None => {
                    if let Some(peer_desc) = peer_desc {
                        let _ = 
                            peers.actived_peers
                                .insert(
                                    peer_id, 
                                    CachedPeerInfo::new(peer_signature, send_time, ping_sequence)
                                );
                        self.0.stack.cacher_manager().add(&peer_desc);
                        Ok(())
                    } else {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "Missing peer desc."))
                    }
                }
            }
        }?;

        Ok(PingResp {
            session_id: ping.session_id,
            ping_sequence: ping_sequence,
            peer_id: self.0.stack.local_device_id().clone(),
            reverse_endpoint: Some(endpoint_pair.remote().clone()),
        })

    }

    pub async fn try_knock_timeout(&self, now: Timestamp) -> Option<Vec<DeviceId>> {
        let last_knock_time = self.0.last_knock_time.load(Ordering::SeqCst);
        let config = &self.0.stack.config().peer_c_s;
        let drop_maps = 
            if now > last_knock_time && Duration::from_micros(now - last_knock_time) > config.knock_timeout {
                let mut peers = self.0.peers.lock().await;
                let mut knocked_peers = Default::default();
                std::mem::swap(&mut knocked_peers, &mut peers.actived_peers);
                std::mem::swap(&mut knocked_peers, &mut peers.knocked_peers);
                self.0.last_knock_time.store(now, Ordering::SeqCst);

                Some(knocked_peers.into_keys().collect())
            } else {
                None
            };

        drop_maps
    }

    pub async fn find_peer(&self, id: &DeviceId, reason: Option<FindPeerReason>) -> Option<FoundPeer> {
        if let Some(_) = 
            self.0.peers
                .lock().await
                .find_peer(id, reason.unwrap_or(FindPeerReason::Other)) {
            self.0.stack
                .cacher_manager()
                .get(id)
                .await
                .map(| device | {
                    FoundPeer {
                        desc: device,
                    }
                })
        } else {
            None
        }
    }
}

#[async_trait::async_trait]
impl PackageEventTrait<(Ping, Signature)> for PeerManager {
    async fn on_package_event(
        &self,
        tunnel: DynamicTunnel,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        data: (Ping, Signature),
    ) -> NearResult<()> {
        // let header_meta = HeaderMeta::new(head, head_ext, Some(tunnel.clone_as_interface()))?;
        let (ping_data, peer_signature) = data;

        let endpoint_pair = {
            let local = head_ext.from.creator_local().ok_or_else(|| {
                error!("missing local endpoint...");
                NearError::new(
                    ErrorCode::NEAR_ERROR_MISSING_DATA,
                    "missing local endpoint.",
                )
            })?;
            let remote = head_ext.from.creator_remote().ok_or_else(|| {
                error!("missing remote endpoint...");
                NearError::new(
                    ErrorCode::NEAR_ERROR_MISSING_DATA,
                    "missing remote endpoint.",
                )
            })?;

            EndpointPair::new(local.clone(), remote.clone())
        };

        let r = self.peer_heartbeat(ping_data, peer_signature, endpoint_pair).await?;

        let package = 
            PackageBuilder::build_head(head.sequence().clone(), None)
                .build_topic(
                    None,
                    self.0.stack.local_device_id().clone(),
                    head_ext.from.requestor.clone(),
                    None,
                )
                .build_body(AnyNamedRequest::with_pingresp(r))
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

#[async_trait::async_trait]
impl PackageEventTrait<(CallReq, Signature)> for PeerManager {
    async fn on_package_event(
        &self,
        tunnel: DynamicTunnel,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        context: (CallReq, Signature),
    ) -> NearResult<()> {

        let (call_req, _singature) = context;
        let (package_source, _target, _) = head_ext.split();
        let from_peer_id = &package_source.requestor;
        let call_sequence = call_req.call_sequence;
        let call_session_id = call_req.session_id;
        let sequence = head.sequence();
        let log_key = format!(
            "[call {}->{} seq({})]",
            from_peer_id,
            call_req.to_peer_id,
            sequence,
        );
        info!("{}.", log_key);

        let endpoints_to_string = | eps: &[Endpoint] | -> String {
            let mut s = "[".to_string();
            if let Some((first, last)) = eps.split_first() {
                s += first.to_string().as_str();

                last.iter()
                    .for_each(| it | {
                        s += ",";
                        s += it.to_string().as_str();
                    });
            }
            s += "]";
            s
        };

        let call_result =
            if let Some(to_peer_cache) = self.find_peer(&call_req.to_peer_id, Some(FindPeerReason::CallFrom(call_req.call_time))).await {
                // Self::call_stat_contract(to_peer_cache, &call_req);
                let from_peer = 
                    if call_req.fromer.is_none() {
                        self.find_peer(from_peer_id, Some(FindPeerReason::CallFrom(call_req.call_time))).await.map(|c| c.desc)
                    } else {
                        call_req.fromer
                    };

                if let Some(from_peer) = from_peer {
                    info!(
                        "[{}] to-peer found, endpoints: {}, always_call: {}.",
                        log_key,
                        endpoints_to_string(to_peer_cache.desc.body().content().endpoints().as_slice()),
                        true,
                    );

                    match self.0.stack
                            .build_package(BuildPackageV1{
                                creator: package_source.creator.clone(),
                                remote: Some(from_peer.object_id().clone()),
                                body: AnyNamedRequest::with_called(CalledReq{
                                    peer_info: from_peer,
                                    call_sequence: call_req.call_sequence,
                                    call_time: call_req.call_time,
                                }),
                                need_sign: true,
                                ..Default::default()
                            })
                            .await {
                        Ok((sequence, package)) => {
                            self.0.stack
                                .tunnel_manager()
                                .post_message((
                                    call_req.to_peer_id.clone(), 
                                    sequence,
                                    package,
                                ))
                                .await
                                .map(|_| {
                                    info!("[{log_key}] successfully post message to {}, sequence: {}", call_req.to_peer_id, sequence);
                                    to_peer_cache
                                })
                                .map_err(| err | {
                                    error!("[{log_key}] failure post message to {}, sequence: {}, with err: {}", call_req.to_peer_id, sequence, err);
                                    err
                                })
                        }
                        Err(err) => {
                            error!("[{log_key}] failure build call-req message to {}, with err: {}", call_req.to_peer_id, err);
                            Err(err)
                        }
                    }
                } else {
                    let error_string = format!("[{}] to-peer not found.", log_key);
                    warn!("{}", error_string);
                    Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string))
                }
            } else {
                warn!("[{}] without from-desc.", log_key);
                Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "not found"))
            };

        let call_resp = 
            match call_result {
                Ok(to_peer) => {
                    CallResp {
                        session_id: call_session_id,
                        call_sequence: call_sequence,
                        result: ErrorCode::NEAR_ERROR_SUCCESS.into_u16() as u8,
                        to_peer_info: Some(to_peer.desc), 
                    }
                }
                Err(err) => {
                    CallResp {
                        session_id: call_session_id,
                        call_sequence: call_sequence,
                        result: err.into_errno() as u8,
                        to_peer_info: None, 
                    }
                }
            };

        // build call-resp package
        match self.0.stack
                .build_package(BuildPackageV1 {
                    creator: package_source.creator,
                    remote: Some(from_peer_id.clone()),
                    sequence: Some(head.sequence().clone()),
                    body: AnyNamedRequest::with_callresp(call_resp),
                    need_sign: true,
                    ..Default::default()
                })
                .await {
            Ok((sequence, package)) => {
                self.0.stack
                    .tunnel_manager()
                    .post_message((
                        tunnel,
                        sequence,
                        package
                    ))
                    .await
                    .map(| _ | {
                        info!("[{}], successfully post (call-resp) message to {}, sequence: {}", log_key, from_peer_id, head.sequence());
                    })
            }
            Err(err) => {
                error!("[{}], failure post (call-resp) message to {}, sequence: {}, with error: {}",
                    log_key,
                    from_peer_id,
                    head.sequence(),
                    err
                );
                Err(err)
            }
        }

        // self.0.stack
        //     .tunnel_manager()
        //     .post_message((
        //         tunnel,

        //     ))
    }
}

#[async_trait::async_trait]
impl PackageEventTrait<(CalledResp, Signature)> for PeerManager {
    async fn on_package_event(
        &self,
        _tunnel: DynamicTunnel,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        data: (CalledResp, Signature),
    ) -> NearResult<()> {
        Ok(())
    }
}

impl OnTimeTrait for PeerManager {
    fn on_time_escape(&self, now: Timestamp) {
        let this = self.clone();
        async_std::task::spawn(async move {
            if let Some(_drops) = this.try_knock_timeout(now).await {

            }
        });
        // if let Some(drops) = self.peer_manager().try_knock_timeout(now) {
        //     for device in &drops {
        //         self.key_store().reset_peer(device)
        //     }
        // }

        // self.resend_queue().try_resend(now);
        // self.0.call_stub.recycle(now);
        // {
        //     let tracker = &mut self.call_tracker;
        //     if let Ordering::Greater = now.duration_since(tracker.begin_time).cmp(&TRACKER_INTERVAL) {
        //         tracker.calls.clear();
        //         tracker.begin_time = now;
        //     }
        // }

        // {
        //     if service.is_stopped() {
        //         return;
        //     }
        //     service.clean_timeout_resource();
        // }
        // task::sleep(Duration::from_micros(100000)).await;
    }
}
