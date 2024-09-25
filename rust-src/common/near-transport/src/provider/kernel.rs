
use std::sync::Arc;

use near_base::*;

use crate::EventHandleTrait;

use super::itf::{ItfWrap};

struct KernelProviderImpl {
    // event_handle: DynamicNetEventHandleTrait,
}

#[derive(Clone)]
pub struct KernelProvider(Arc<KernelProviderImpl>);

impl KernelProvider {
    pub fn new(/* event_handle: DynamicNetEventHandleTrait */) -> Self {
        Self(Arc::new(KernelProviderImpl{
            /* event_handle: event_handle */
        }))
    }
}

#[async_trait::async_trait]
impl EventHandleTrait for KernelProvider {
    async fn on_package_event(&self, from: ObjectId, command: u16, sequence: u32, data: &[u8]) -> NearResult<ItfWrap> {
        
    }
}

