use std::{
    collections::BTreeMap, sync::{atomic::{AtomicU64, Ordering}, Arc}, time::Duration
};

use async_std::sync::Mutex;
use log::{error, info, trace, warn};

use near_base::{device::DeviceId, *};

use crate::{
    coturn::{p::ProxyStub, stun::p::{BaseEventManager, CallTemplate}}, 
    h::OnTimeTrait, 
    network::DataContext, 
    process::PackageFailureTrait, 
    tunnel::DynamicTunnel, 
    CreatorMeta, 
    RequestorMeta,
};
use crate::package::*;
use crate::Stack;

// pub struct FoundPeer {
//     pub desc: DeviceObject,
// }

struct CachedPeerInfo {
    tunnels: Vec<(Timestamp, DynamicTunnel)>,

    last_send_time: Timestamp,
    last_call_time: Timestamp,
    #[allow(unused)]
    last_checkout_time: Timestamp,
}

impl CachedPeerInfo {
    fn new(
        tunnel: DynamicTunnel,
        send_time: Timestamp, 
    ) -> CachedPeerInfo {
        CachedPeerInfo {
            tunnels: vec![(now(), tunnel)],
            last_send_time: send_time,
            last_call_time: 0,
            last_checkout_time: 0,
        }
    }

    fn update_tunnel(&mut self, tunnel: DynamicTunnel) {
        let now = now();
        let tunnels = std::mem::replace(&mut self.tunnels, vec![]);
        let r: Vec<(Timestamp, DynamicTunnel)> = 
            tunnels.into_iter()
                .filter(| (_timestamp, t) | !t.ptr_eq(&tunnel))
                .collect();

        let new_tunnels = [vec![(now, tunnel)], r].concat();

        let _ = std::mem::replace(&mut self.tunnels, new_tunnels);
    }

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

    pub async fn try_knock_timeout(&self, now: Timestamp) -> Option<Vec<DeviceId>> {
        let last_knock_time = self.0.last_knock_time.load(Ordering::SeqCst);
        let config = &self.0.stack.config().peer_c_s;
        let drop_maps = 
            if now > last_knock_time && Duration::from_micros(now - last_knock_time) > config.knock_timeout {
                let mut peers = self.0.peers.lock().await;
                let mut knocked_peers = Default::default();
                std::mem::swap(&mut knocked_peers, &mut peers.actived_peers);

                // knock timeout tunnel
                knocked_peers.values_mut()
                    .for_each(| peer | {
                        let _ = peer.tunnels.drain(1..);
                    });

                std::mem::swap(&mut knocked_peers, &mut peers.knocked_peers);
                self.0.last_knock_time.store(now, Ordering::SeqCst);

                Some(knocked_peers.into_keys().collect())
            } else {
                None
            };

        drop_maps
    }

    pub async fn find_peer(
        &self, 
        id: &DeviceId, 
        reason: Option<FindPeerReason>
    ) -> NearResult<DynamicTunnel> {
        match self.0.peers
                .lock().await
                .find_peer(id, reason.unwrap_or(FindPeerReason::Other)) {
            Some(peer) => {
                peer.tunnels.first()
                    .map(| (_, tunnel) | tunnel.clone())
                    .ok_or_else(|| {
                        NearError::new(ErrorCode::NEAR_ERROR_UNACTIVED, format!("{id} unactived tunnel."))
                    })
            }
            None => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("not found {id}")))
            }
        }
    }

}

impl PeerManager {
    
    pub(crate) async fn on_bind_request(
        &self,
        _head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        tunnel: DynamicTunnel,
        data: (StunReq, Signature),
    ) -> NearResult<StunReq> {
        let peer_id = head_ext.requestor();
        let (ping, peer_signature) = data;

        if let StunType::PingRequest = ping.stun_type() {
        } else {
            debug_assert!(false, "not ping requesut");
        }
        
        let endpoint_pair = {
            let local = head_ext.from.creator_local().ok_or_else(|| {
                error!("missing local endpoint.");
                NearError::new(
                    ErrorCode::NEAR_ERROR_MISSING_DATA,
                    "missing local endpoint.",
                )
            })?;
            let remote = head_ext.from.creator_remote().ok_or_else(|| {
                error!("missing remote endpoint.");
                NearError::new(
                    ErrorCode::NEAR_ERROR_MISSING_DATA,
                    "missing remote endpoint.",
                )
            })?;

            EndpointPair::new(local.clone(), remote.clone())
        };

        let resp = 
            match {
                let exist_cache_found = 
                    | cached_peer: &mut CachedPeerInfo, tunnel: DynamicTunnel, new_signature: Signature | -> NearResult<()> {
                        if cached_peer.last_send_time > new_signature.sign_time() {
                            log::warn!("ping send-time little.");
                            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "ping send-time little."))
                        } else {
                            Ok(())
                        }?;

                        cached_peer.update_tunnel(tunnel);
                        cached_peer.last_send_time = new_signature.sign_time();

                        Ok(())
                    };

                let peers = &mut *self.0.peers.lock().await;

                // 1.从活跃peer中搜索已有cache
                if let Some(p) = peers.actived_peers.get_mut(peer_id) {
                    exist_cache_found(p, tunnel, peer_signature)
                } else {
                    // 2.从待淘汰peer中搜索已有cache
                    match peers.knocked_peers.remove(peer_id) {
                        Some(mut cached_found) => {
                            if let Err(err) = exist_cache_found(&mut cached_found, tunnel, peer_signature) {
                                Err(err)
                            } else {
                                let _ = peers.actived_peers.insert(peer_id.clone(), cached_found);
                                Ok(())
                            }
                        }
                        None => {
                            let _ = 
                                peers.actived_peers
                                    .insert(
                                        peer_id.clone(), 
                                        CachedPeerInfo::new(tunnel, peer_signature.sign_time())
                                    );
                            Ok(())
                        }
                    }
                }

            } {
            Ok(_) => {
                StunReq::new(StunType::PingResponse)
                    .set_mapped_address(Some(endpoint_pair.remote().clone()))
            }
            Err(err) => {
                StunReq::new(StunType::PingErrorResponse)
                    .set_error_code(Some(err))
            }
        };

        Ok(resp)

        // PostMessageTrait::post_message(
        //     &self.0.stack, 
        //     (
        //         Some(tunnel.clone()),
        //         RequestorMeta {
        //             sequence: Some(head.sequence().clone()),
        //             creator: Some(CreatorMeta { creator: head_ext.creator().cloned(), ..Default::default() }),
        //             to: Some(head_ext.from.requestor.clone()),
        //             need_sign: true,
        //             ..Default::default()
        //         },
        //         AnyNamedRequest::with_stun(resp),
        //         None,
        //     ))
        //     .await
        //     .map(| _ | {
        //         info!("successfully post ping-resp sequence: {}", head.sequence());
        //         ()
        //     })
        //     .map_err(| err | {
        //         error!("failed post ping-resp package to {} with {}", tunnel, err);
        //         err
        //     })

    }

    pub async fn on_call_request(
        &self,
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        context: (StunReq, Signature),
    ) -> NearResult<StunReq> {
        let call_time = now();
        let (mut call, _peer_signature) = context;
        let package_source = head_ext.source();
        // let (package_source, _target, _) = head_ext.split();
        let sequence = head.sequence();

        if let StunType::CallRequest = call.stun_type() {
        } else {
            debug_assert!(false, "not call requesut");
        }

        let to_peer_id = 
            call.take_target().ok_or_else(|| {
                error!("on_call_request: missing to_peer_id, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing peer id")
            })?;
        let fromer = 
            call.take_fromer().ok_or_else(|| {
                error!("on_call_request: missing fromer, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing fromer")
            })?;
        let fromer_id = fromer.object_id();

        let log_key = format!("[call {fromer_id}->{to_peer_id} seq({sequence})]", );
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

        let this = self.clone();
        let call_result =
            async_std::task::block_on(async move {
                let log_key_ref = &log_key;
                let peer_tunnel = 
                    this.find_peer(&to_peer_id, Some(FindPeerReason::CallFrom(call_time)))
                        .await?;

                let from_peer = fromer;

                info!(
                    "[{log_key_ref}] to-peer found, endpoints: {}, always_call: {}.",
                    endpoints_to_string(from_peer.body().content().endpoints().as_slice()),
                    true,
                );

                let (called_resp, _) = {
                    CallTemplate::<StunReq>::call(
                        self.0.stack.clone(),
                        Some(peer_tunnel),
                        RequestorMeta {
                            sequence: Some(sequence.clone()),
                            creator: Some(CreatorMeta {
                                creator: package_source.creator.clone(),
                                ..Default::default()
                            }),
                            to: Some(to_peer_id.clone()),
                            need_sign: true,
                            ..Default::default()
                        },
                        AnyNamedRequest::with_stun(
                            StunReq::new(StunType::CallRequest)
                                .set_fromer(Some(from_peer))
                                .set_target(Some(to_peer_id.clone()))
                        ),
                        Some(self.0.stack.config().peer_c_s.invite_timeout),
                    )
                    .await
                    .map(| fut | {
                        info!("[{log_key_ref}] successfully post message to {}, sequence: {}", to_peer_id, sequence);
                        fut
                    })
                    .map_err(| err | {
                        error!("[{log_key_ref}] failure post message to {}, sequence: {}, with err: {}", to_peer_id, sequence, err);
                        err
                    })?
                };

                info!("[{log_key_ref}], called resp: {called_resp}", );
                Ok(called_resp)
        });

        Ok(
            match call_result {
                Ok(call_resp) => call_resp,
                Err(err) => {
                    StunReq::new(StunType::CallErrorResponse)
                        .set_error_code(err)
                }
            }
        )

//         PostMessageTrait::post_message(
//                 &self.0.stack, 
//                 (
//                     Some(tunnel),
//                     RequestorMeta {
//                         sequence: Some(head.sequence().clone()),
//                         creator: Some(CreatorMeta {
//                             creator: package_source.creator,
//                             ..Default::default()
//                         }),
//                         need_sign: true,
//                         ..Default::default()
//                     },
//                     AnyNamedRequest::with_callresp(call_resp),
//                     None
//                 )
//             )
//             .await
//             .map(| _ | {
//                 info!("[{}], successfully post (call-resp) message to {}, sequence: {}", log_key, from_peer_id, head.sequence());
//             })
//             .map_err(| err | {
//                 error!("[{}], failure post (call-resp) message to {}, sequence: {}, with error: {}",
//                     log_key,
//                     from_peer_id,
//                     head.sequence(),
//                     err
//                 );
//                 err
//             })
//     }
    }

    pub async fn on_call_response(
        &self,
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        data: (StunReq, Signature),
    ) -> NearResult<()> {

        let (call_resp, _peer_signature) = data;

        match call_resp.stun_type() {
            StunType::CallResponse | StunType::CallErrorResponse => {},
            _ => debug_assert!(false, "not call response")
        };

        trace!(
            "PeerManager::on_called_response: head: {head}, head_ext: {head_ext}, CallResp: {call_resp}, begin...",
        );

        BaseEventManager::get_instance()
            .take_routine(
                head_ext.requestor(), 
                head.sequence(), 
                0
            )
            .ok_or_else(|| {
                warn!("not found event callback, sequence: {}", head.sequence());
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found event callback")
            })?
            .emit(head, head_ext, call_resp.into())
            .await

    }

    pub async fn on_allocation_request(
        &self,
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        _tunnel: DynamicTunnel,
        data: (StunReq, Signature),
    ) -> NearResult<StunReq> {
        let (mut allocation, _peer_signature) = data;

        match allocation.stun_type() {
            StunType::AllocationChannelRequest => {},
            _ => debug_assert!(false, "not allocaltion request")
        };

        trace!(
            "PeerManager::on_allocation_request: head: {head}, head_ext: {head_ext}, AllocationRequest: {allocation}, begin...",
        );

        let turn_config = &self.0.stack.config().turn_config;
        let now = now();
        let sequence = head.sequence();
        let package_source = head_ext.source();
        let target = 
            allocation.take_target().ok_or_else(|| {
                let error_string = format!("missing target item");
                error!("{error_string}, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, error_string)
            })?;

        let creator = 
            head_ext.creator().ok_or_else(|| {
                let error_string = format!("missing creator item");
                error!("{error_string}, sequence: {sequence}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, error_string)
            })?;

        let log_key = format!("[call {creator}->{target} seq({sequence})]", );
        info!("{}.", log_key);

        let this = self.clone();
        let result =
            async_std::task::block_on(async move {
                let log_key_ref = &log_key;
                let target_tunnel = 
                    this.find_peer(&target, Some(FindPeerReason::CallFrom(now)))
                        .await?;

                let proxy_stub = ProxyStub::new();

                let _ = 
                    self.0.stack
                        .turn_server()
                        .create_tunnel(proxy_stub.channel_key().clone(), (creator.clone(), target.clone()));

                // let proxy_stub = 
                //     self.0.stack
                //         .turn_stub_cachers().unwrap()
                //         .new_proxy(creator.clone(), target.clone())
                //         .map_err(| err | {
                //             error!("{log_key} failed create proxy stub");
                //             err
                //         })?;

                let (mut allocation_resp, _) = {
                    CallTemplate::<StunReq>::call(
                        self.0.stack.clone(),
                        Some(target_tunnel),
                        RequestorMeta {
                            sequence: Some(sequence.clone()),
                            creator: Some(CreatorMeta {
                                creator: package_source.creator.clone(),
                                ..Default::default()
                            }),
                            to: Some(target.clone()),
                            need_sign: true,
                            ..Default::default()
                        },
                        AnyNamedRequest::with_stun(
                            StunReq::new(StunType::AllocationChannelRequest)
                                .set_target(Some(target.clone()))
                                .set_mixhash(Some(proxy_stub.channel_key().clone()))
                                .set_live_minutes(Some(turn_config.mixhash_live_minutes))
                                .set_proxy_address(self.0.stack.turn_server().external_host().cloned())
                        ),
                        Some(self.0.stack.config().peer_c_s.invite_timeout),
                    )
                    .await
                    .map(| fut | {
                        info!("[{log_key_ref}] successfully post message to {target}, sequence: {sequence}", );
                        fut
                    })
                    .map_err(| err | {
                        error!("[{log_key_ref}] failure post message to {target}, sequence: {sequence}, with err: {err}", );
                        err
                    })?
                };

                info!("[{log_key_ref}], called resp: {allocation_resp}", );

                match allocation_resp.stun_type() {
                    StunType::AllocationChannelErrorResponse => {
                        Err(allocation_resp.take_error_code().unwrap_or(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, "unknown")))
                    }
                    StunType::AllocationChannelResponse => {
                        Ok(proxy_stub)
                    }
                    _ => unreachable!("won't reach here")
                }
        });

        Ok(
            match result {
                Ok(proxy_stub) => {
                    StunReq::new(StunType::AllocationChannelResponse)
                        .set_mixhash(Some(proxy_stub.into_channel_key()))
                        .set_live_minutes(Some(turn_config.mixhash_live_minutes))
                        .set_proxy_address(self.0.stack.turn_server().external_host().cloned())
                }
                Err(err) => {
                    StunReq::new(StunType::AllocationChannelErrorResponse)
                        .set_error_code(Some(err))
                }
            }
        )

    }

    pub async fn on_allocation_response(
        &self,
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        data: (StunReq, Signature),
    ) -> NearResult<()> {

        let (allocation_resp, _peer_signature) = data;

        match allocation_resp.stun_type() {
            StunType::AllocationChannelResponse | StunType::AllocationChannelErrorResponse => {},
            _ => debug_assert!(false, "not allocation response")
        };

        trace!(
            "PeerManager::on_allocation_response: head: {head}, head_ext: {head_ext}, AllocationChannelResponse: {allocation_resp}, begin...",
        );

        BaseEventManager::get_instance()
            .take_routine(
                head_ext.requestor(), 
                head.sequence(), 
                0
            )
            .ok_or_else(|| {
                warn!("not found event callback, sequence: {}", head.sequence());
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found event callback")
            })?
            .emit(head, head_ext, allocation_resp.into())
            .await

    }


}

#[async_trait::async_trait]
impl PackageFailureTrait for PeerManager {

    async fn on_package_failure(
        &self, 
        error: NearError,
        data: DataContext,
    ) {
        let sequence = data.head.sequence();
        let target = data.head_ext.to();
        let major_command = data.head.major_command();

        trace!("PeerManager::on_package_failure, sequence: {sequence}, major_command: {major_command}, target: {target}, error: {error}");

        if let Some(event) = BaseEventManager::get_instance().take_routine(target, sequence, 0) {
            event.emit_error(error, data).await;
        } else {
            warn!("not found event callback, sequence: {sequence}");
        }
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
