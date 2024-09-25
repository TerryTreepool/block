
use std::{
        collections::{hash_map::Entry, HashMap, }, 
        sync::{Arc, RwLock}, 
    };

use log::trace;
use near_base::{aes_key::KeyMixHash, device::DeviceId, Endpoint, NearResult};

use crate::coturn::turn::p::{ProxyDatagramTrait, ProxyInterface};
use super::{tunnel::Tunnel, TunnelRef};

struct ProxyManagerImpl {
    interface: Option<ProxyInterface>,
    tunnel_mixhash_map: RwLock<HashMap<KeyMixHash, TunnelRef>>,
}

#[derive(Clone)]
pub struct ProxyManager(Arc<ProxyManagerImpl>);

impl std::fmt::Display for ProxyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProxyManager")
    }
}

impl ProxyManager {
    pub fn open(local: Option<Endpoint>) -> NearResult<Self> {
        // TODO: 支持多interface扩展
        let this = Self(Arc::new(ProxyManagerImpl {
            interface: None,
            tunnel_mixhash_map: RwLock::new(HashMap::new()),
        }));

        let interface = ProxyInterface::open(local, Box::new(this.clone()) as Box<dyn ProxyDatagramTrait>)?;

        {
            unsafe { 
                &mut *(Arc::as_ptr(&this.0) as *mut ProxyManagerImpl)
            }
            .interface = Some(interface);
        }

        Ok(this)
    }

    pub fn tunnel_of(&self, mix_hash: &KeyMixHash) -> Option<TunnelRef> {
        self.0.tunnel_mixhash_map.read().unwrap().get(mix_hash).cloned()
    }

    pub fn erase_tunnel(&self, mix_hash: &KeyMixHash) {
        let _ = 
            self.0.tunnel_mixhash_map.write().unwrap()
                .remove(mix_hash);
    }

    pub fn create_tunnel(&self, mix_hash: KeyMixHash, device_pair: (DeviceId, DeviceId)) -> TunnelRef {

        match self.0.tunnel_mixhash_map.write().unwrap().entry(mix_hash) {
            Entry::Occupied(existed) => { existed.get().clone() },
            Entry::Vacant(empty) => {
                let tunnel = TunnelRef::new(Tunnel::new(device_pair));
                empty.insert(tunnel.clone());
                tunnel
            }
        }

    }

    pub fn tunnels(&self) -> Vec<(KeyMixHash, TunnelRef)> {
        self.0.tunnel_mixhash_map.read().unwrap()
            .iter()
            .map(| (k, v) | (k.clone(), v.clone()) )
            .collect()
    }

    pub fn endpoint(&self) -> Option<&Endpoint> {
        self.0.interface.as_ref().map(| interface | interface.local() )
    }
}

#[async_trait::async_trait]
impl ProxyDatagramTrait for ProxyManager {

    async fn on_proxied_datagram(
        &self, 
        mix_hash: KeyMixHash, 
        datagram: &[u8], 
        from: Endpoint
    ) {
        trace!("ProxyManager::on_proxied_datagram(): mix_hash: {mix_hash}, from: {from}");

        if let Some(proxy_to) = 
            if let Some(tunnel) = self.tunnel_of(&mix_hash) {
                tunnel.on_proxied_datagram(mix_hash, from)
            } else {
                None
            } {
            let _ = self.0.interface.as_ref().unwrap().send_to(datagram, &proxy_to).await;
        }
    }
}