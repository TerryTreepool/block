
use std::time::Duration;
use std::{sync::Arc, path::PathBuf};

use common::{RuntimeProcessTrait, RuntimeStack, RoutineTemplate};
use log::{trace, error, info};
use near_base::builder_codec_macro::Empty;
use near_core::get_data_path;

use near_base::{NearResult, ErrorCode, NearError};
use protos::hci::schedule::{Schedule_list, Schedule_info};
use topic_util::topics::hci_schedule::*;
use topic_util::topics::hci_storage::{NEAR_THING_STORAGE_SCHEDULE_QUERYALL_PUB, NEAR_THING_STORAGE_SCHEDULE_QUERY_PUB};

use crate::caches::schedule_manager;
use crate::routines::schedule::add::AddSchduleRoutine;
use crate::routines::schedule::execute::ExecuteSchuleRoutine;
use crate::routines::schedule::remove::RemoveSchduleRoutine;

struct Config {
    #[allow(unused)]
    work_path: PathBuf,
    polling_interval: Duration,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            work_path: Default::default(),
            polling_interval: std::time::Duration::from_secs(30),
        }
    }
}

struct ProcessComponent {
    // things_manager: thing_components::ThingCollect,
    schedule_manager: schedule_manager::Manager,
}

struct ProcessImpl {
    #[allow(unused)]
    service_name: String,
    #[allow(unused)]
    config: Config,
    components: Option<ProcessComponent>,
}

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

impl Process {
    pub fn new(service_name: &str) -> Self {
        let ret = Self(Arc::new(ProcessImpl{
            service_name: service_name.to_owned(),
            config: Config {
                work_path: get_data_path().join(service_name),
                ..Default::default()
            },
            components: None,
        }));

        unsafe {&mut *(Arc::as_ptr(&ret.0) as *mut ProcessImpl) }
            .components = 
                Some(ProcessComponent {
                    schedule_manager: schedule_manager::Manager::new(),
                });

        ret
    }

    #[inline]
    pub fn schedule_manager(&self) -> &schedule_manager::Manager {
        &self.0.components.as_ref().unwrap().schedule_manager
    }

    pub(self) async fn subscribe_topic(&self) -> NearResult<()> {

        {
            // add schedule
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_SCHEDULE_ADD_PUB.topic(),
                    move || {
                        Ok(AddSchduleRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // remove schedule
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_SCHEDULE_REMOVE_PUB.topic(),
                    move || {
                        Ok(RemoveSchduleRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // remove schedule
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_SCHEDULE_EXECUTE_PUB.topic(),
                    move || {
                                        Ok(ExecuteSchuleRoutine::new(arc_self.clone()))
                                    })?;
        }

        Ok(())
    }

    async fn sync_all_group_maindata(&self) -> NearResult<Vec<Schedule_info>> {
        let routine = 
            RoutineTemplate::<Schedule_list>::call(
                NEAR_THING_STORAGE_SCHEDULE_QUERYALL_PUB.topic().clone(),
                Empty
            )
            .await
            .map_err(| e | {
                error!("{e}");
                e
            })?;

        async_std::future::timeout(std::time::Duration::from_secs(5), routine)
            .await
            .map_err(| _e |{
                NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, "timeout")
            })?
            .map(| mut v | {
                v.take_schedules()
            })

    }

    async fn init_data(&self) -> NearResult<()> {
        let schedules = loop {
            match self.sync_all_group_maindata().await {
                Ok(groups) => break(Ok(groups)),
                Err(e) => {
                    match e.errno() {
                        ErrorCode::NEAR_ERROR_TIMEOUT => continue,
                        _ => break(Err(e))
                    }
                }
            }
        }
        .map_err(| e | {
            error!("failed init group data with err: {e}");
            e
        })?;

        let mut fut = vec![];
        for schedule in schedules.iter() {
            fut.push(
                RoutineTemplate::<Schedule_info>::call(
                    NEAR_THING_STORAGE_SCHEDULE_QUERY_PUB.topic().clone(),
                    schedule.schedule_id()
                )
                .await
                .map_err(| e | {
                    error!("{e}");
                    e
                })?
            );
        }

        let res = futures::future::join_all(fut).await;
        for (index, item) in res.into_iter().enumerate() {

            trace!("group thing info: {index}-{:?}", item);

            // let schedule = schedules.get_mut(index).unwrap();

            match item {
                Ok(schedule_info) => {
                    let schedule_id = schedule_info.schedule_id.clone();
                    let _ = 
                        self.schedule_manager().update_schedule(schedule_info)
                            .map(| _ | {
                                info!("Successfully add [{schedule_id}] schedule.");
                            })
                            .map_err(| e | {
                                error!("failed add [{schedule_id}] schedule with err:{e}.");
                                e
                            });
                }
                Err(e) => {
                    error!("{e}");
                }
            }
        }

        Ok(())
    }

    pub(self) fn start(&self) {
        trace!("on_time_escape");
        let arc_self = self.clone();
        let polling_interval = arc_self.0.config.polling_interval;

        async_std::task::spawn(async move {
            loop {
                arc_self.schedule_manager().on_time_escape();

                let _ = async_std::future::timeout(polling_interval, async_std::future::pending::<()>()).await;
            }
        });
    }

}

unsafe impl Sync for Process {}
unsafe impl Send for Process {}

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {

    async fn run(&self) -> NearResult<()> {
        trace!("run...");

        self.subscribe_topic().await?;

        self.init_data().await?;

        self.start();

        Ok(())
    }

    fn quit(&self) {
        trace!("quiting...");
    }
    
}
