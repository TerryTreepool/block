use std::{
    collections::BTreeMap, 
    time::Duration, 
    sync::{atomic::{AtomicU64, Ordering}, Mutex}
};

use near_base::{device::DeviceId, Timestamp, DeviceObject, EndpointPair, NearResult, Signature, NearError, ErrorCode, now, AesKey};

#[derive(Copy, Clone)]
pub struct Config {
    pub ping_interval_init: Duration,
    pub ping_interval: Duration,

    pub offline: Duration,

    pub knock_timeout: Duration,    // default is 5 minutes

    pub invite_interval: Duration,
    pub invite_timeout: Duration,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            ping_interval_init: Duration::from_millis(500),
            ping_interval: Duration::from_millis(25000),
            offline: Duration::from_millis(300000),
            knock_timeout: Duration::from_secs(300),
            invite_interval: Duration::from_millis(200),
            invite_timeout: Duration::from_millis(3000),
        }
    }
}

pub struct FoundPeer {
    pub desc: DeviceObject,
    pub sender: EndpointPair,
}


struct CachedPeerInfo {
    desc: DeviceObject,
    last_signature: Signature,
    last_endpoint: EndpointPair,
    last_key: Option<AesKey>,
    last_send_time: Timestamp,
    last_call_time: Timestamp,
    last_checkout_time: Timestamp,
    last_ping_sequence: u64,
}

impl CachedPeerInfo {
    fn new(
        desc: DeviceObject, 
        last_signature: Signature,
        key: AesKey, 
        endpoint_pair: EndpointPair,
        send_time: Timestamp, 
        ping_sequence: u64, 
    ) -> CachedPeerInfo {
        CachedPeerInfo {
            last_ping_sequence: ping_sequence,
            last_signature,
            last_key: Some(key),
            last_endpoint: endpoint_pair,
            desc,
            last_send_time: send_time,
            last_call_time: 0,
            last_checkout_time: 0,
        }
    }

    fn to_found_peer(&self) -> FoundPeer {
        FoundPeer {
            desc: self.desc.clone(), 
            sender: self.last_endpoint.clone(), 
        }
    }

    fn update_key(&mut self, key: AesKey) {
        if let Some(k) = self.last_key.as_mut() {
            *k = key;
        } else {
            self.last_key = Some(key);
        }
    }


    fn update_desc(&mut self, desc: &DeviceObject, new_signature: &Signature) -> NearResult<()> {

        match new_signature.sign_time().cmp(&self.last_signature.sign_time()) {
            std::cmp::Ordering::Equal => Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, "signature time equal, ignore")),
            std::cmp::Ordering::Less => Err(NearError::new(ErrorCode::NEAR_ERROR_EXPIRED, "signature time expire")),
            std::cmp::Ordering::Greater => Ok(())
        }?;

        let _ = std::mem::replace(&mut self.desc, desc.clone());
        let _ = std::mem::replace(&mut self.last_signature, new_signature.clone());
        
        Ok(())
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
                FindPeerReason::Checkout(t) => {
                    if t > found_peer_cache.last_checkout_time {
                        found_peer_cache.last_checkout_time = t;
                    }
                    Some(found_peer_cache)
                }
                FindPeerReason::Other => {
                    Some(found_peer_cache)
                }
            }
        } else {
            None
        }
    }
}


pub struct PeerManager {
    peers: Mutex<Peers>, 
    last_knock_time: AtomicU64,
    config: Config,
}

pub enum FindPeerReason {
    Checkout(Timestamp),
    CallFrom(Timestamp),
    Other,
}


impl PeerManager {
    pub fn new(config: Option<Config>) -> PeerManager {
        PeerManager {
            peers: Mutex::new(Peers {
                actived_peers: Default::default(),
                knocked_peers: Default::default(),
            }),
            last_knock_time: AtomicU64::new(now()),
            config: config.unwrap_or_default(),
        }
    }

    pub fn peer_heartbeat(
        &self, 
        peer_desc: DeviceObject, 
        peer_signature: Signature,
        peer_key: AesKey,
        endpoint_pair: EndpointPair,
        send_time: Timestamp, 
        ping_sequence: u64,
    ) -> bool {

        let peer_id = peer_desc.object_id().clone();

        let exist_cache_found = | cached_peer: &mut CachedPeerInfo | -> bool {
            if cached_peer.last_send_time > send_time {
                log::warn!("ping send-time little.");
                return false;
            }

            if let Err(e) = cached_peer.update_desc(&peer_desc, &peer_signature) {
                log::warn!("ping update device-info failed, err: {}", e);

                match e.errno() {
                    ErrorCode::NEAR_ERROR_IGNORE => {},
                    _ => { return false; }
                }
            }

            cached_peer.last_send_time = send_time;
            cached_peer.last_ping_sequence = ping_sequence;
            cached_peer.last_endpoint = endpoint_pair.clone();

            cached_peer.update_key(peer_key);

            true
        };

        let peers = &mut *self.peers.lock().unwrap();
        // 1.从活跃peer中搜索已有cache
        if let Some(p) = peers.actived_peers.get_mut(&peer_id) {
            return exist_cache_found(p);
        }

        // 2.从待淘汰peer中搜索已有cache
        let to_active = 
            match peers.knocked_peers.get_mut(&peer_id) {
                Some(p) => {
                    if !exist_cache_found(p) {
                        return false
                    }
                    peers.knocked_peers.remove(&peer_id)
                }
                None => {
                    None
                }
            };

        if let Some(to_active) = to_active {
            assert!(peers.actived_peers.insert(peer_id, to_active).is_none());
            return true;
        } else {
            peers.actived_peers.insert(
                peer_id, 
                CachedPeerInfo::new(peer_desc, 
                                          peer_signature, peer_key, endpoint_pair, send_time, ping_sequence));
            return true;
        }
    }

    pub fn try_knock_timeout(&self, now: Timestamp) -> Option<Vec<DeviceId>> {
        let last_knock_time = self.last_knock_time.load(Ordering::SeqCst);
        let drop_maps = 
            if now > last_knock_time && Duration::from_micros(now - last_knock_time) > self.config.knock_timeout {
                let mut peers = self.peers.lock().unwrap();
                let mut knocked_peers = Default::default();
                std::mem::swap(&mut knocked_peers, &mut peers.actived_peers);
                std::mem::swap(&mut knocked_peers, &mut peers.knocked_peers);
                self.last_knock_time.store(now, Ordering::SeqCst);

                Some(knocked_peers.into_keys().collect())
            } else {
                None
            };

        drop_maps
    }

    pub fn find_peer(&self, id: &DeviceId, reason: Option<FindPeerReason>) -> Option<FoundPeer> {
        self.peers
            .lock().unwrap()
            .find_peer(id, reason.unwrap_or(FindPeerReason::Other))
            .map(|c| c.to_found_peer())
    }
}