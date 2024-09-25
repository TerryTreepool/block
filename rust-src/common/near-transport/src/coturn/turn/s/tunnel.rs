
use std::sync::atomic::AtomicU64;

use crossbeam::epoch::{self, Atomic, Owned, Shared};
use log::trace;
use near_base::{aes_key::KeyMixHash, device::DeviceId, now, Endpoint, Timestamp};

#[allow(unused)]
pub struct ProxyDeviceStubRef<'a> {
    pub(super) id: &'a DeviceId,
    pub(super) endpoint: *const Endpoint,
    pub(super) last_active: Timestamp, 
}

pub struct ProxyDeviceStub {
    pub(super) id: DeviceId, 
    pub(super) endpoint: Atomic<Endpoint>,
    pub(super) last_active: AtomicU64, 
}

pub struct Tunnel {
    active_time: Timestamp,
    device_stub_pair: (ProxyDeviceStub, ProxyDeviceStub),     // mix_key: AesKey,
    // mixhash: Vec<MixHashInfo>,
}

impl std::fmt::Display for Tunnel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tunnel")
    }
}

impl Tunnel {
    pub fn new(device_pair: (DeviceId, DeviceId)) -> Self {
        let now = now();

        Self {
            active_time: now, 
            device_stub_pair: (
                ProxyDeviceStub {
                    id: device_pair.0, endpoint: Atomic::default(), last_active: AtomicU64::new(now),
                },
                ProxyDeviceStub {
                    id: device_pair.1, endpoint: Atomic::default(), last_active: AtomicU64::new(now),
                },
            )
        }
    }

    pub fn active_time(&self) -> Timestamp {
        self.active_time
    }

    #[allow(unused)]
    pub fn stub_pair<'a>(&'a self) -> (ProxyDeviceStubRef<'a>, ProxyDeviceStubRef<'a>) {
        let guard = &epoch::pin();

        {
            (
                ProxyDeviceStubRef {
                    id: &self.device_stub_pair.0.id,
                    endpoint: self.device_stub_pair.0.endpoint.load(std::sync::atomic::Ordering::SeqCst, guard).as_raw(),
                    last_active: self.device_stub_pair.0.last_active.load(std::sync::atomic::Ordering::SeqCst),
                },
                ProxyDeviceStubRef {
                    id: &self.device_stub_pair.1.id,
                    endpoint: self.device_stub_pair.0.endpoint.load(std::sync::atomic::Ordering::SeqCst, guard).as_raw(),
                    last_active: self.device_stub_pair.1.last_active.load(std::sync::atomic::Ordering::SeqCst),
                }

            )
        }
    }

    pub fn on_proxied_datagram(&self, mix_hash: KeyMixHash, from: Endpoint) -> Option<Endpoint> {

        let now = now();
        let guard = &epoch::pin();

        if self.device_stub_pair.0.endpoint.compare_exchange(
                Shared::null(),
                    Owned::new(from.clone()),
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                    guard
            ).is_ok() {
            self.device_stub_pair.0.last_active.store(now, std::sync::atomic::Ordering::SeqCst);
            trace!("{self} mix_hash:{mix_hash} update 0 endpoint pair to {from}");
            None
        } else if self.device_stub_pair.1.endpoint.compare_exchange(
                    Shared::null(),
                    Owned::new(from.clone()),
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                    guard
        ).is_ok() {
            self.device_stub_pair.1.last_active.store(now, std::sync::atomic::Ordering::SeqCst);
            trace!("{self} mix_hash:{mix_hash} update 1 endpoint pair to {from}");
            None
        } else {
            let left = &self.device_stub_pair.0;
            let right = &self.device_stub_pair.1;

            unsafe {

                let left_endpoint = 
                    match left.endpoint.load(std::sync::atomic::Ordering::SeqCst, guard).as_ref() {
                        Some(ep) => ep.clone(),
                        None => unreachable!()
                    };
                let right_endpoint = 
                    match right.endpoint.load(std::sync::atomic::Ordering::SeqCst, guard).as_ref() {
                        Some(ep) => ep.clone(),
                        None => unreachable!()
                    };

                if left_endpoint.eq(&from) {
                    left.last_active.store(now, std::sync::atomic::Ordering::SeqCst);
                    Some(right_endpoint)
                } else if right_endpoint.eq(&from) {
                    right.last_active.store(now, std::sync::atomic::Ordering::SeqCst);
                    Some(left_endpoint)
                } else {
                    trace!("ProxyTunnel mix_hash:{mix_hash} mix_hash not found endpoint pair from ({from})");
                    None
                }
            }
        }

    }

}
