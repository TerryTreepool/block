
use std::{sync::Arc, path::PathBuf};

use log::debug;
use near_base::{DeviceObject, ErrorCode, FileDecoder, NearError, NearResult, ObjectId, ObjectTypeCode, ServiceObjectSubCode, };
use near_core::{get_service_path, get_data_path};
use near_util::TopicRef;
use near_transport::{ProcessTrait, RoutineEventTrait, };

use common::{RuntimeProcessTrait, CoreStack};

use crate::event::Manager as EventManager;

#[derive(Clone)]
pub(super) struct Config {
    #[allow(unused)]
    work_path: PathBuf,
}

struct ProcessComponents {
    event_manager: EventManager,
}

struct ProcessImpl {
    service_name: String,
    config: Config,

    components: Option<ProcessComponents>,
}

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

unsafe impl Send for Process {}
unsafe impl Sync for Process {}

impl Process {
    pub fn new(service_name: &str) -> NearResult<Box<Self>> {
        let config = Config {
            work_path: get_service_path(service_name),
        };

        let ret = Self(Arc::new(ProcessImpl{
            service_name: service_name.to_owned(),
            config: config.clone(),
            components: None,
        }));

        let mut_ret = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ProcessImpl) };
        mut_ret.components = Some(ProcessComponents {
            event_manager: EventManager::new(ret.clone()),
        });

        Ok(Box::new(ret))
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn service_name(&self) -> &str {
        &self.0.service_name
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn config(&self) -> &Config {
        &self.0.config
    }

    #[inline]
    pub(crate) fn event_manager(&self) -> &EventManager {
        &self.0.components.as_ref().unwrap().event_manager
    }

}

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {
    async fn run(&self) -> NearResult<()> {
        let need_find_sn = 
            match CoreStack::get_instance().stack().local_device_id().object_type_code()? {
                ObjectTypeCode::Device(_) => true,
                _ => false,
            };

        if need_find_sn {
            let dir = 
                std::fs::read_dir(get_data_path())
                    .map_err(| e | {
                        let error_string = format!("failed found {} with err: {e}.", get_data_path().display());
                        NearError::new(ErrorCode::NEAR_ERROR_FATAL, error_string)
                    })?;

            for file in dir {
                if let Ok(file) = file {
                    if {
                        if let Some(extension_name) = file.path().extension() {
                            extension_name.eq_ignore_ascii_case("desc")
                        } else {
                            false
                        }
                    } {
                        match DeviceObject::decode_from_file(file.path().as_path()) {
                            Ok(o) => {
                                if let Ok(codec) = o.object_id().object_type_code() {
                                    debug!("search {} is {}", file.path().as_path().display(), codec);
                                    match codec {
                                        ObjectTypeCode::Service(codec) if codec == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_COTURN_MINER as u8 => {
                                            CoreStack::get_instance().stack().add_sn(o).await?;
                                        }
                                        _ => { /* ignore */ }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn quit(&self) {
        
    }
}

impl ProcessTrait for Process {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        self.event_manager().create_routine(from, topic)
    }
}
