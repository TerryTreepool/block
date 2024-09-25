
use std::{sync::{Arc, RwLock, },
    };
use base::{ModuleTrait, TOPIC_SUBSCRIBE_STATIC, MessageExpire, };
use libloading::{Library, };

use log::{info, error};
use near_base::{ExtentionObject, ObjectId, };
use near_base::{DeviceObject,
                NearResult, ErrorCode, NearError,
    };
use near_core::path_utils::get_extention_path;
use near_core::*;
use near_transport::{StackOpenParams, StackRuntimeParams, Stack, ProcessTrait, RoutineEventTrait, topic::{TopicRef, }, };
use base::SubscribeMessage;

use crate::config::RUNTIME_CONFIG;

struct RuntimeStackImpl {
    core: DeviceObject,
    local: ExtentionObject,
    module: Option<Box<dyn ModuleTrait>>,
    lib: Library,
}

lazy_static::lazy_static! {
    pub static ref RUNTIME_STACK: RwLock<Option<Stack>> = RwLock::new(None);
}

#[derive(Clone)]
pub struct RuntimeStack(Arc<RuntimeStackImpl>);

impl RuntimeStack {
    pub async fn open(core: DeviceObject, local: ExtentionObject) -> NearResult<Self> {
        let plugin_name = local.desc().content().get_extention_name();
        let lib = unsafe {
            Library::new(get_extention_path(plugin_name).as_path()).
                map_err(| err | NearError::new(ErrorCode::NEAR_ERROR_3RD, err.to_string()) )
        }?;

        let runtime_stack =
            Self(Arc::new(RuntimeStackImpl {
                            core, local,
                            module: None,
                            lib,
            }));

        let stack_open_params = StackOpenParams {
            config: None,
            // process_event: Some(Box::new(RuntimeProcess::new(runtime_stack.clone()))),
        };

        // init stack
        let stack = {
            let stack = &mut *RUNTIME_STACK.write().unwrap();
            match stack {
                Some(stack) => { stack.clone() },
                None => {
                    let new_stack = Stack::open_runtime(StackRuntimeParams {
                                                core_service: runtime_stack.0.core.clone(),
                                                local_extention: runtime_stack.0.local.clone(),
                                                runtime_process_impl: Box::new(RuntimeProcess::new(runtime_stack.clone())),
                                            },
                                            stack_open_params)
                                        .await?;
                    *stack = Some(new_stack.clone());
                    new_stack
                }
            }
        };

        runtime_stack.init_stack()?;

        // init module impl
        let module = {
            let func: libloading::Symbol<fn(stack: *const Stack) -> *const Box<dyn ModuleTrait>> = unsafe {
                runtime_stack.0.lib.get(b"create_extention_module")
                    .map_err(| err | {
                        NearError::new(ErrorCode::NEAR_ERROR_3RD, err.to_string())
                    })
            }.unwrap();
            let module = func(&stack as *const Stack);
            unsafe { (*module).clone_as_module() }
        };

        let stack_impl = unsafe { &mut * (Arc::as_ptr(&runtime_stack.0) as *mut RuntimeStackImpl) };
        stack_impl.module = Some(module);

        Ok(runtime_stack)
    }

    // pub async fn start(&self) -> NearResult<()> {


    //     Ok(Module(Arc::new(ModuleImpl{
    //         stack
    //     })))

    //     let runtime_module = &mut *RUNTIME_MODULE.write().unwrap();
    //     if let Some(_) = runtime_module {
    //         Ok(())
    //     } else {
    //         Module::open(local, core)
    //             .await
    //             .map(| m | {
    //                 *runtime_module = Some(m.clone());
    //             })
    //     }

    //     // let func: libloading::Symbol<fn(local_extention: *const ExtentionObject, core_device: *const DeviceObject)> = unsafe {
    //     //     self.0.lib.get(b"create_extention_module")
    //     //         .map_err(| err | {
    //     //             NearError::new(ErrorCode::NEAR_ERROR_3RD, err.to_string())
    //     //         })
    //     // }?;

    //     // let local = self.0.local.clone();
    //     // let core = self.0.core.clone();

    //     // func(&local as *const ExtentionObject, &core as *const DeviceObject);

    //     Ok(())
    // }
    pub(super) fn module(&self) -> &dyn ModuleTrait {
        self.0.module.as_ref().unwrap().as_ref()
    }

    pub fn local(&self) -> &ExtentionObject {
        &self.0.local
    }

    pub fn remote(&self) -> &DeviceObject {
        &self.0.core
    }

}

impl RuntimeStack {
    fn init_stack(&self) -> NearResult<()> {
        let arc_self = self.clone();

        async_std::task::block_on(async move {
            let stack = {
                RUNTIME_STACK.read().unwrap().as_ref().unwrap().clone()
            };

            let mut message_list = vec![];
            arc_self.local()
                .desc()
                .content()
                .get_subscribe_messages()
                .iter()
                .for_each(| item | message_list.push((item.clone(), MessageExpire::Forever)));

            if message_list.len() > 0 {
                info!("Will subscribe topic: {:?}", message_list);

                match stack.post_message(None,
                                        TOPIC_SUBSCRIBE_STATIC.clone(),
                                        SubscribeMessage { message_list },
                                        None) {
                    Ok(_) => {
                        Ok(())
                    }
                    Err(err) => {
                        error!("Failed subscribe topic with {}", err);
                        Err(err)
                    }
                }
            } else {
                Ok(())
            }

            // match async_std::future::timeout(RUNTIME_CONFIG.wait_online_timeout(), stack.wait_online()).await {
            //     Ok(online) => {
            //         info!("runtime is {}", online.to_string());

            //         if message_list.len() > 0 {
            //             info!("Will subscribe topic: {:?}", message_list);

            //             match stack.post_message(None,
            //                                     TOPIC_SUBSCRIBE_STATIC.clone(),
            //                                     SubscribeMessage { message_list },
            //                                     None) {
            //                 Ok(_) => {
            //                     Ok(())
            //                 }
            //                 Err(err) => {
            //                     error!("Failed subscribe topic with {}", err);
            //                     Err(err)
            //                 }
            //             }
            //         } else {
            //             Ok(())
            //         }
            //     }
            //     Err(err) => {
            //         let error_message = format!("failed init stack by wait online timeout {} with err = {}", RUNTIME_CONFIG.wait_online_timeout().as_secs(), err);

            //         error!("{}", error_message);

            //         Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_message))
            //     }
            // }
        })
    }
}

struct RuntimeProcessImpl {
    stack: RuntimeStack,
}

#[derive(Clone)]
struct RuntimeProcess(Arc<RuntimeProcessImpl>);

impl RuntimeProcess {
    fn new(stack: RuntimeStack) -> Self {
        Self (Arc::new(RuntimeProcessImpl { stack }))
    }
}

impl ProcessTrait for RuntimeProcess {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    // fn create_routine(&self, from: &ObjectId, topic: &str) -> NearResult<Box<dyn RoutineEventTrait>>;
    // fn online_routine(&self) -> Option<Vec<u8>> {
    //     unimplemented!()
    //     // let message = SubscribeMessage {
    //     //     message_list: self.stack.local().desc().content().get_subscribe_messages().clone()
    //     // };
    //     // let mut text = vec![0u8; message.raw_capacity()];
    //     // let r =
    //     // match message.serialize(text.as_mut_slice()) {
    //     //     Ok(_) => {
    //     //         Some(text)
    //     //     }
    //     //     Err(err) => {
    //     //         error!("failed serialize message, with error: {}", err);
    //     //         None
    //     //     }
    //     // };

    //     // r
    // }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        self.0.stack.module().create_routine(from, topic)
    }
}
