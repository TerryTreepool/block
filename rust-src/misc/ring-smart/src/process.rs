
use std::{path::PathBuf, sync::Arc};

use log::trace;

use near_base::{NearResult, ObjectId};
use near_core::get_service_path;
use near_transport::{ProcessTrait, RoutineEventTrait};
use near_util::TopicRef;

use common::{RuntimeProcessTrait, RuntimeStack};
use storage::{StorageTrait, sqlite_storage::SqliteStorage};
use topic_util::topics::ring_smart::*;

use crate::{routines::{publish::PublishRoutine, check_out::CheckOutRoutine}, cahces::DeviceObjectItem};

#[derive(Clone)]
#[allow(unused)]
pub(crate) struct Config {
    pub(crate) work_path: PathBuf,
}

struct ProcessComponents {
    device_storage: Box<dyn StorageTrait<DeviceObjectItem>>,
}

struct ProcessImpl {
    #[allow(unused)]
    service_name: String,
    #[allow(unused)]
    config: Config,
    storage: SqliteStorage,

    components: Option<ProcessComponents>,
}

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

impl Process {
    pub async fn new(service_name: &str) -> NearResult<Box<Self>> {
        let config = {
            let work_path = get_service_path(service_name);
            Config {
                work_path: work_path,
            }    
        };

        let ret = Self(Arc::new(ProcessImpl{
            service_name: service_name.to_owned(),
            config: config.clone(),
            storage: SqliteStorage::new(config.work_path.join(format!("{service_name}.db")).as_path())?,
            components: None,
        }));

        let mut_ret = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ProcessImpl) };
        mut_ret.components = Some(ProcessComponents {
            device_storage: ret.0.storage.add_storage("device").await?,
        });

        Ok(Box::new(ret))
    }

    #[inline]
    pub(crate) fn device_storage(&self) -> &dyn StorageTrait<DeviceObjectItem> {
        self.0.components.as_ref().unwrap().device_storage.as_ref()
    }
}

impl Process {
    pub(in self) async fn subscribe_topic(&self) -> NearResult<()> {
        {
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    CORE_RING_CHAIN_PUBLISH_PUB.topic(), 
                    move || Ok(PublishRoutine::new(arc_self.clone()))
                )?;
        }

        {
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    CORE_RING_CHAIN_CHECKOUT_PUB.topic(), 
                    move || Ok(CheckOutRoutine::new(arc_self.clone()))
                )?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {

    async fn run(&self) -> NearResult<()> {
        trace!("run enter");

        self.subscribe_topic().await?;

        Ok(())
    }

    fn quit(&self) {
        trace!("quiting...");
    }

}

impl ProcessTrait for Process {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, sender: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        trace!("from: {}, topic: {}, ", sender, topic);

        RuntimeStack::get_instance()
            .topic_routine_manager()
            .call(topic)
    }

}

