
use std::sync::Arc;

use log::info;
use near_transport::process::ProcessEventTrait;
use once_cell::sync::OnceCell;

use near_base::{ObjectId, PrivateKey};
use near_base::{DeviceObject,
                NearResult,
    };
use near_transport::{StackOpenParams, Stack, ProcessTrait, RoutineEventTrait, StackServiceParams, };
use near_util::TopicRef;

use crate::RuntimeProcessTrait;

static CORE_INSTANCE: OnceCell<CoreStack> = OnceCell::new();

pub(crate) struct CoreStackImpl {
    core: DeviceObject,
    stack: Option<Stack>,
    process: Option<CoreProcess>,
}

#[derive(Clone)]
pub struct CoreStack(Arc<CoreStackImpl>);

impl CoreStack {
    pub fn get_instance() -> &'static Self {
        CORE_INSTANCE.get().expect("core must init.")
    }

    pub async fn open(core: DeviceObject, 
                      core_private_key: PrivateKey,
                      runtime_process_impl: Box<dyn RuntimeProcessTrait>,
                      process_impl: Box<dyn ProcessTrait>
    ) -> NearResult<CoreProcess> {

        let core_stack =
            Self(Arc::new(CoreStackImpl {
                            core,
                            stack: None,
                            process: None,
            }));

        let stack_open_params = StackOpenParams {
            config: None,
            device_cacher: None,
        };

        let core_process = CoreProcess::new(core_stack.clone(), runtime_process_impl, process_impl);

        // init stack
        let stack = 
            Stack::open_service(
                StackServiceParams {            
                        core_service: core_stack.0.core.clone(),
                        core_service_private_key: core_private_key,
                        sn_service: vec![],
                        service_process_impl: core_process.clone_as_process(),
                    },
                    stack_open_params
                )
                .await?;
        let core_stack_mut = unsafe { &mut *(Arc::as_ptr(&core_stack.0) as *mut CoreStackImpl) };
        core_stack_mut.stack = Some(stack);
        core_stack_mut.process = Some(core_process.clone());

        CORE_INSTANCE.set(core_stack).unwrap();

        Ok(core_process)
    }

    #[allow(unused)]
    pub(crate) fn quit(&self) {
        if let Some(process) = self.0.process.as_ref() {
            process.quit();
        }
    }
}

impl std::fmt::Debug for CoreStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CoreStack")
    }
}

impl CoreStack {

    #[inline]
    pub fn stack(&self) -> Stack {
        self.0.stack.as_ref().unwrap().clone()
    }

    #[allow(unused)]
    pub fn remote(&self) -> &DeviceObject {
        &self.0.core
    }

}

struct CoreProcessImpl {
    #[allow(unused)]
    stack: CoreStack,
    runtime_process_impl: Box<dyn RuntimeProcessTrait>,
    process_impl: Box<dyn ProcessTrait>,
}

#[derive(Clone)]
pub struct CoreProcess(Arc<CoreProcessImpl>);

impl CoreProcess {
    pub(crate) fn new(
        stack: CoreStack, 
        runtime_process_impl: Box<dyn RuntimeProcessTrait>, 
        process_impl: Box<dyn ProcessTrait>
    ) -> Self {
        Self (Arc::new(CoreProcessImpl {
            stack,
            runtime_process_impl,
            process_impl,
        }))
    }
}

#[async_trait::async_trait]
impl RuntimeProcessTrait for CoreProcess {
    async fn run(&self) -> NearResult<()> {
        // init process
        self.0.runtime_process_impl.run().await?;

        async_std::task::block_on(async_std::future::pending::<()>());

        Ok(())        
    }
    
    fn quit(&self) {
        self.0.runtime_process_impl.quit();
    }
}

impl ProcessTrait for CoreProcess {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        self.0.process_impl.create_routine(from, topic)
    }
}

impl ProcessEventTrait for CoreProcess {
    fn on_reinit(&self) {
        info!("FATAL!!!!!!!!!!!!: unimplemented CoreProcess::on_reinit");
    }
}
