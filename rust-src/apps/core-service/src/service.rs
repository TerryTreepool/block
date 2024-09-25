
use std::sync::{Arc, RwLock};

use near_base::{DeviceObject, ObjectId, 
                NearResult, };
use near_transport::{Stack, StackOpenParams, StackServiceParams, ProcessTrait, RoutineEventTrait, };
use near_util::TopicRef;

use crate::event::Manager as EventManager;

struct ServiceStackImpl {
    event_manager: Option<EventManager>,
}

#[derive(Clone)]
pub struct ServiceStack(Arc<ServiceStackImpl>);

lazy_static::lazy_static!{
    pub static ref CORE_STACK: RwLock<Option<Stack>> = RwLock::new(None);

}

impl ServiceStack {
    pub async fn open(local_device: DeviceObject) -> NearResult<Self> {
        let params = StackOpenParams {
            config: None,
        };

        let ret = Self(Arc::new(ServiceStackImpl {
            event_manager: None,
        }));

        let stack = {
            let core_stack = &mut *CORE_STACK.write().unwrap();
            match core_stack {
                Some(stack) => stack.clone(),
                None => {
                    let stack = Stack::open_service(StackServiceParams{
                                    core_service: local_device, 
                                    service_process_impl: Box::new(Processor::new(ret.clone())),
                                }, params).await?;
                    *core_stack = Some(stack.clone());
                    stack
                }
            }
        };

        let event_manager = EventManager::new(stack.clone());
        let r = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ServiceStackImpl) };
        r.event_manager = Some(event_manager);

        Ok(ret)
    }

}

impl ServiceStack {
    pub(super) fn event_manager(&self) -> &EventManager {
        self.0.event_manager.as_ref().unwrap()
    }

}

struct ProcessorImpl {
    service: ServiceStack,
}

#[derive(Clone)]
pub struct Processor(Arc<ProcessorImpl>);

impl Processor {
    fn new(service: ServiceStack) -> Self {
        Self(Arc::new(ProcessorImpl{
            service,
        }))
    }
}

impl ProcessTrait for Processor {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        self.0.service.event_manager().create_routine(from, topic)
    }
}
