
use std::sync::Arc;

use base::MessageType;
use log::{info, error, trace};
use near_base::ErrorCode;
use near_base::NearError;
use near_transport::process::ProcessEventTrait;
use near_transport::RequestorMeta;
use once_cell::sync::OnceCell;

use near_base::{ExtentionObject, DeviceObject, ObjectId, NearResult, };
use near_transport::{StackOpenParams, StackRuntimeParams, Stack, ProcessTrait, RoutineEventTrait, };
use near_util::{TopicRef, Topic, TOPIC_CORE_SUBSCRIBE, TOPIC_CORE_DISSUBSCRIBE};

use base::SubscribeMessage;
use base::MessageExpire;
use protos::core_message::Message;
use protos::core_message::message::Message_type;
use protos::core_message::{Subscribe_message, Dissubscribe_message};
use topic_util::topic_types::TOPIC_SUBSCRIBE_STATIC;

use crate::RuntimeProcessTrait;
use crate::topic_runtime::manager::{Manager as TopicRouineManager, TopicRoutineOpEventTrait};

static RUNTIME_INSTANCE: OnceCell<RuntimeStack> = OnceCell::new();

struct RuntimeStackComponents {
    topic_routine_manager: TopicRouineManager,
    stack: Stack,
    process: RuntimeProcess,
    // let topic_routine_manager = TopicRouineManager::new(Box::pin(ret.clone()));
}
pub(crate) struct RuntimeStackImpl {
    core: DeviceObject,
    local: ExtentionObject,
    components: Option<RuntimeStackComponents>,
}

#[derive(Clone)]
pub struct RuntimeStack(Arc<RuntimeStackImpl>);

impl RuntimeStack {
    pub fn get_instance() -> &'static Self {
        RUNTIME_INSTANCE.get().expect("runtime must init.")
    }

    pub async fn open(core: DeviceObject, 
                      local: ExtentionObject, 
                      process: Box<dyn RuntimeProcessTrait>) -> NearResult<RuntimeProcess> {

        let runtime_stack =
            Self(Arc::new(RuntimeStackImpl {
                            core, local,
                            components: None,
            }));

        let stack_open_params = StackOpenParams {
            config: None,
            device_cacher: None,
        };

        let runtime_process = RuntimeProcess::new(runtime_stack.clone(), process);

        // init stack
        let stack = 
            Stack::open_runtime(StackRuntimeParams {
                                                core_service: runtime_stack.0.core.clone(),
                                                local_extention: runtime_stack.0.local.clone(),
                                                runtime_process_impl: runtime_process.clone_as_process(),
                                                runtime_process_event_impl: Some(Box::new(runtime_process.clone()) as Box<dyn ProcessEventTrait>),
                                            },
                                            stack_open_params)
                                        .await?;
        let runtime_stack_mut = unsafe { &mut *(Arc::as_ptr(&runtime_stack.0) as *mut RuntimeStackImpl) };

        runtime_stack_mut.components = Some(RuntimeStackComponents {
            topic_routine_manager: TopicRouineManager::new(Box::pin(runtime_stack.clone())),
            stack,
            process: runtime_process.clone(),
        });

        RUNTIME_INSTANCE.set(runtime_stack).unwrap();

        Ok(runtime_process)
    }

    #[allow(unused)]
    pub(crate) fn quit(&self) {
        self.0.components.as_ref().unwrap().process.quit();
    }
}

impl std::fmt::Debug for RuntimeStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RuntimeStack")
    }
}

impl RuntimeStack {

    #[inline]
    pub fn stack(&self) -> Stack {
        self.0.components.as_ref().unwrap().stack.clone()
    }

    #[inline]
    pub fn topic_routine_manager(&self) -> &TopicRouineManager {
        &self.0.components.as_ref().unwrap().topic_routine_manager
    }

    #[allow(unused)]
    pub fn local(&self) -> &ExtentionObject {
        &self.0.local
    }

    #[allow(unused)]
    pub fn remote(&self) -> &DeviceObject {
        &self.0.core
    }

}

impl RuntimeStack {
    pub(crate) async fn init_stack(&self) -> NearResult<()> {
        let stack = self.stack();

        let mut message_list = vec![];
        self.local()
            .body()
            .content()
            .subscribe_messages()
            .iter()
            .for_each(| item | message_list.push((item.clone(), MessageExpire::Forever)));

        if message_list.len() > 0 {
            info!("Will subscribe topic: {:?}", message_list);

            match stack.post_message(
                        RequestorMeta {
                            topic: Some(TOPIC_SUBSCRIBE_STATIC.clone()),
                            ..Default::default()
                        },
                        SubscribeMessage { message_list },
                        None
                    )
                    .await {
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
    }
}

impl TopicRoutineOpEventTrait for RuntimeStack {
    fn subscribe_message(&self, topic: &Topic, expire: MessageExpire, mt: Option<MessageType>) -> NearResult<()> {

        async_std::task::block_on(async move {

            trace!("subscribe_message: topic: {topic}, expire: {:?}", expire);

            let mut message = Subscribe_message::new();
            message.messge = vec![Message{
                                    message: topic.clone().into(), 
                                    mt: {
                                        match mt.unwrap_or_default() {
                                            MessageType::Public => Message_type::Public,
                                            MessageType::Private => Message_type::Private,
                                        }.into()
                                    },
                                    ..Default::default()
                                }];

            let message = 
                protos::RawObjectHelper::encode_with_raw(message)
                .map_err(| e | {
                    let error_string = format!("failed build Subscribe_message with err = {e}");
                    error!("{error_string}");
                    e
                })?;

            self.stack()
                .post_message(
                    RequestorMeta { 
                        topic: Some(TOPIC_CORE_SUBSCRIBE.topic().clone()), 
                        ..Default::default()
                    },
                    message, 
                    None
                )
                .await
        })
    }

    fn dissubscribe_message(&self, topic: &Topic) -> NearResult<()> {

        async_std::task::block_on(async move {
            trace!("dissubscribe_message: topic: {topic}");

            let mut message = Dissubscribe_message::new();
            message.message_name = topic.clone().into();

            let message = 
                protos::RawObjectHelper::encode_with_raw(message)
                .map_err(| e | {
                    let error_string = format!("failed build Subscribe_message with err = {e}");
                    error!("{error_string}");
                    e
                })?;

            self.stack()
                .post_message(
                    RequestorMeta { 
                        topic: Some(TOPIC_CORE_DISSUBSCRIBE.topic().clone()),
                        ..Default::default()
                    },
                    message, 
                    None
                )
                .await
        })

    }
}

struct RuntimeProcessImpl {
    stack: RuntimeStack,
    process_impl: Box<dyn RuntimeProcessTrait>,
}

#[derive(Clone)]
pub struct RuntimeProcess(Arc<RuntimeProcessImpl>);

impl RuntimeProcess {
    pub(crate) fn new(stack: RuntimeStack, process_impl: Box<dyn RuntimeProcessTrait>) -> Self {
        Self (Arc::new(RuntimeProcessImpl {
            stack,
            process_impl,
        }))
    }

}

#[async_trait::async_trait]
impl RuntimeProcessTrait for RuntimeProcess {
    async fn run(&self) -> NearResult<()> {
        // init stack
        self.0.stack.init_stack().await?;

        if self.0.stack.stack().wait_online().await {
            info!("online");
            Ok(())
        } else {
            error!("offline");
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNACTIVED, "Timed out when connect remote"))
        }?;

        // init process
        self.0.process_impl.run().await?;

        async_std::task::block_on(async_std::future::pending::<()>());

        Ok(())
    }

    fn quit(&self) {
        self.0.process_impl.quit();
    }
}

impl ProcessTrait for RuntimeProcess {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        trace!("from: {}, topic: {}, ", from, topic);

        self.0.stack
            .topic_routine_manager()
            .call(topic)
    }
}

impl ProcessEventTrait for RuntimeProcess {
    fn on_reinit(&self) {
        self.0.stack
            .topic_routine_manager()
            .reregister_topic_event();
    }
}
