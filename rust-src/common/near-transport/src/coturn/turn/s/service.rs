
use std::{net::{IpAddr, SocketAddr}, sync::Arc, time::Duration};

use log::debug;
use near_base::{aes_key::KeyMixHash, device::DeviceId, now, Endpoint, NearResult};

use crate::Stack;

use super::proxy::ProxyManager;
use super::TunnelRef;

struct ServiceImp {
    stack: Stack,
    proxy_manager: ProxyManager, 
    external_host: Option<Endpoint>,
}

#[derive(Clone)]
pub struct Service(Arc<ServiceImp>);

impl Service {
    pub fn open(stack: Stack, external_host: Option<SocketAddr>) -> NearResult<Self> {

        let proxy_manager = ProxyManager::open(None)?;
        let mut proxy_address = proxy_manager.endpoint().cloned().unwrap();
        let external_host = 
            if let Some(external_host) = external_host {
                match external_host.ip() {
                    IpAddr::V4(host) => {
                        if let Some(sockaddr) = proxy_address.mut_addr() {
                            sockaddr.set_ip(IpAddr::V4(host));
                        }

                        Some(proxy_address)
                    }
                    IpAddr::V6(_host) => { None }
                }
            } else {
                Some(proxy_address)
            };

        let this = Self(Arc::new(ServiceImp {
            stack,
            proxy_manager,
            external_host,
        }));

        {
            let this = this.clone();
            async_std::task::spawn(async move {
                this.timer().await;
            });
        }

        Ok(this)
    }

    #[inline]
    pub fn external_host(&self) -> Option<&Endpoint> {
        self.0.external_host.as_ref()
    }

    pub fn create_tunnel(&self, mix_hash: KeyMixHash, device_pair: (DeviceId, DeviceId)) -> TunnelRef {
        self.0.proxy_manager.create_tunnel(mix_hash, device_pair)
    }

    async fn timer(&self) {
        let now = now();
        let config = &self.0.stack.config().turn_config;

        loop {
            for (mixhash, tunnel) in self.0.proxy_manager.tunnels() {
                if config.mixhash_live_minutes as u64 + tunnel.active_time() > now {
                    // timeout, erase
                    debug!("{mixhash} timeout, will erase.", );
                    self.0.proxy_manager.erase_tunnel(&mixhash);
                }
            }

            let _ = async_std::future::timeout(Duration::from_secs(60), async_std::future::pending::<()>()).await;
        }
    }

}
