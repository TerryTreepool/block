
use std::{sync::{Arc, RwLock}, collections::{BTreeMap, btree_map::Entry}};

use near_base::{NearResult, EndpointPair};

use crate::{network::{TcpInterface, DynamicInterface}, Stack, package::DynamicPackage};

use super::compose::Compose;

struct ManagerImpl {
    stack: Stack,

    compose_array: RwLock<BTreeMap<EndpointPair, Compose>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new(stack: Stack) -> Self {
        Self(Arc::new(ManagerImpl {
            stack,
            compose_array: Default::default(),
        }))
    }

    pub fn compose_of(&self, ep_pair: &EndpointPair) -> Option<Compose> {
        self.0.compose_array.read().unwrap()
            .get(&ep_pair)
            .cloned()            
    }

    pub fn create_compose(&self, ep_pair: EndpointPair, interface: DynamicInterface) -> Compose {
        match self.compose_of(&ep_pair) {
            Some(compose) => { compose }
            None => {
                let new_compose = Compose::new(interface);

                let array = &mut *self.0.compose_array.write().unwrap();
                 match array.entry(ep_pair) {
                    Entry::Occupied(found) => { found.get().clone() }
                    Entry::Vacant(empty) => {
                        empty.insert(new_compose.clone());
                        new_compose
                    }
                }
            }
        }

    }
}

impl Manager {

    fn on_tcp_package(&self, interface: TcpInterface, package: DynamicPackage) -> NearResult<()> {
        self.create_compose(interface.endpoint_pair(), DynamicInterface::new(interface))
            .on_package(package)
    }

}
