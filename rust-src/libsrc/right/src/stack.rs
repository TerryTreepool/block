
use std::sync::Arc;

use near_base::NearResult;
use near_transport::{Stack as BaseStack};

use crate::RightStackConfig;

struct StackComponents {

}

struct StackImpl {
    service_name: String,
    stack: BaseStack,
    config: RightStackConfig,

    components: Option<StackComponents>,
}

#[derive(Clone)]
pub struct Stack(Arc<StackImpl>);

impl Stack {
    pub fn open(service_name: &str, stack: BaseStack, config: RightStackConfig) -> Self {
        let ret = Self(Arc::new(StackImpl{
            service_name: service_name.to_owned(),
            stack,
            config,
            components: None,
        }));

        ret
    }

    #[allow(unused)]
    #[inline]
    pub fn service_name(&self) -> &str {
        &self.0.service_name
    }

    #[inline]
    pub(crate) fn base_stack(&self) -> &BaseStack {
        &self.0.stack
    }

    #[inline]
    pub(crate) fn config(&self) -> &RightStackConfig {
        &self.0.config
    }

}

impl Stack {
    pub fn grant(&self) -> NearResult<()> {
        unimplemented!()
    }

    
}
