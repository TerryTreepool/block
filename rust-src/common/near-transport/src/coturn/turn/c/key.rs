
use std::{collections::{btree_map::Entry, BTreeMap, HashSet}, sync::{Arc, RwLock}};

use near_base::{aes_key::KeyMixHash, device::DeviceId, now, Endpoint, Timestamp};

pub struct PeerProxyStub {
    pub(super) mix_hash: KeyMixHash,
    pub(super) start_time: Timestamp,
    pub(super) live_minutes: u8,
    pub(super) proxy_address: Endpoint,
}

struct TurnMixHashImpl {
    turn_mixhash_list: RwLock<BTreeMap<DeviceId, Arc<PeerProxyStub>>>,
}

#[derive(Clone)]
pub struct TurnMixHash(Arc<TurnMixHashImpl>);

impl TurnMixHash {
    pub fn new() -> TurnMixHash {
        Self(Arc::new(TurnMixHashImpl{
            turn_mixhash_list: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn append(
        &self, 
        peer_id: DeviceId, 
        mix_hash: KeyMixHash, 
        live_minutes: u8, 
        proxy_address: Endpoint
    ) {
        match self.0.turn_mixhash_list.write().unwrap().entry(peer_id) {
            Entry::Occupied(existed) => {
                let mut_stub = unsafe { &mut *(Arc::as_ptr(existed.get()) as *mut PeerProxyStub) };
                mut_stub.mix_hash = mix_hash;
                mut_stub.start_time = now();
                mut_stub.live_minutes = live_minutes;
                mut_stub.proxy_address = proxy_address;
            }
            Entry::Vacant(empty) => {
                empty.insert(Arc::new(PeerProxyStub{
                    mix_hash,
                    start_time: now(),
                    live_minutes,
                    proxy_address,
                }));
            }
        }
    }

    pub fn get(
        &self,
        peer_id: &DeviceId
    ) -> Option<Arc<PeerProxyStub>> {
        self.0.turn_mixhash_list.read().unwrap().get(peer_id).cloned()
    }

    #[allow(unused)]
    pub fn erase(
        &self,
        peer_id: &DeviceId
    ) {
        let _ = self.0.turn_mixhash_list.write().unwrap().remove(peer_id);
    }

    pub fn is_valid(
        &self,
        peer_id: &DeviceId
    ) -> bool {
        let now = now();
        let mut_stub = &mut *self.0.turn_mixhash_list.write().unwrap();

        if let Some(stub) = mut_stub.get(peer_id) {
            if stub.start_time + stub.live_minutes as u64 > now {
                true
            } else {
                mut_stub.remove(peer_id);
                false
            }
        } else {
            false
        }
    }

    pub fn proxy_addresses(&self) -> Vec<Endpoint> {
        let proxy_addresses: Vec<Endpoint> = {
            self.0.turn_mixhash_list.read().unwrap()
                .iter()
                .map(| (_, v) | {
                    v.proxy_address.clone()
                })
                .collect()
        };

        let proxy_address_set: HashSet<Endpoint> = proxy_addresses.into_iter().collect();

        proxy_address_set.into_iter().collect()
    }
}
