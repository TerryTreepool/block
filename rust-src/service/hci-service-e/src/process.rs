
use std::{sync::Arc, path::PathBuf};

use log::{trace, error};

use near_base::thing::ThingObject;
use near_core::get_app_path;
use near_base::{NearResult, NearError, ErrorCode};

use common::{RuntimeProcessTrait, RuntimeStack, RoutineTemplate};
use protos::hci::thing::{Thing_query_all, Thing_info_list};
use topic_util::topics::hci_service::*;
use topic_util::topics::hci_storage::{NEAR_THING_STORAGE_MULITPLE_THINGOBJECT_QUERY_PUB, NEAR_THING_STORAGE_THING_QUERY_ALL_PUB};

use crate::cache::thing_components::ThingCollect;
use crate::hci::scanning::ScanProcessorEventTrait;
use crate::routines::add_thing_task::AddThingTaskRoutine;
use crate::routines::query_all_thing_task::QueryAllThingTaskRoutine;
use crate::routines::schedule::add::AddScheduleRoutine;
use crate::routines::schedule::execute::ExecuteScheduleRoutine;
use crate::routines::schedule::remove::RemoveScheduleRoutine;
use crate::routines::search_task::SearchRoutine;
use crate::routines::get_task_result::GetTaskResultRoutine;
use crate::routines::crud_thing_task::CrudThingTaskRoutine;
use crate::routines::ctrl_thing_task::ControlThingTaskRoutine;
use crate::routines::Config as RoutinesConfig;
use crate::lua::{manager::Manager as LuaManager, configure::ConfigureData};
use crate::tasks::{manager::{Manager as TaskManager, Config as TaskConfig},
                   cb::TaskManagerCb, 
                   TaskCbTrait, 
                   TaskModule, 
                   TaskCbData};
use crate::hci::{scanning::ScanProcessor, advertising::AdvertisingProcessor};

#[derive(Clone, Default)]
pub struct Config {
    pub work_path: PathBuf,
    pub task_config: TaskConfig,
    pub ctrl_task_config: TaskConfig,
    pub routines_config: RoutinesConfig,
}

struct ProcessComponents {
    task_manager: TaskManager,
    lua_manager: LuaManager,
    task_manager_cb: TaskManagerCb,
    things_components: ThingCollect,
}

struct ProcessImpl {
    service_name: String,
    config: Config,

    components: Option<ProcessComponents>,
}

pub static MAIN_PROCESS: once_cell::sync::OnceCell<Process> = once_cell::sync::OnceCell::new();

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

impl Process {
    pub fn get_instance() -> &'static Self {
        MAIN_PROCESS.get().expect("uninit")
    }

    pub async fn new(service_name: &str, config: Option<Config>) -> Self {
        let ret = Self(Arc::new(ProcessImpl{
            service_name: service_name.to_owned(),
            config: Config {
                work_path: {
                    let work_path = get_app_path().join(service_name);
                    let _ = std::fs::create_dir_all(work_path.as_path()).unwrap();
                    work_path
                },
                task_config: Default::default(),
                ctrl_task_config: TaskConfig {
                    interval: std::time::Duration::from_micros(100),
                    ..Default::default()
                },
                routines_config: config.map(| routine_cfg | routine_cfg.routines_config).unwrap_or_default(),
            },
            components: None,
        }));

        ConfigureData::init();
        let lua_manager = LuaManager::open(ret.0.config.work_path.join("lua")).await.expect("init lua manager");
        let task_manager_cb = TaskManagerCb::new(ret.clone(), Box::new(ret.clone()) as Box<dyn TaskCbTrait>);
        let task_manager = TaskManager::start(ret.clone()).expect("init task manager.");

        {
            let mut_ret = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ProcessImpl) };
            mut_ret.components =
                Some(ProcessComponents{
                    lua_manager,
                    task_manager: task_manager.clone(),
                    task_manager_cb,
                    things_components: Default::default(),
                    // schedule_manager: ScheduleManager::new(ret.clone()),
                });
        }

        MAIN_PROCESS.set(ret.clone()).map_err(| _ | "has been value, don't reinit it.").unwrap();

        ret
    }

    #[inline]
    #[allow(unused)]
    pub fn service_name(&self) -> &str {
        self.0.service_name.as_str()
    }

    #[inline]
    pub fn config(&self) -> &Config {
        &self.0.config
    }

    #[inline]
    pub fn task_manager(&self) -> &TaskManager {
        &self.0.components.as_ref().unwrap().task_manager
    }

    #[inline]
    pub fn task_manager_cb(&self) -> &TaskManagerCb {
        &self.0.components.as_ref().unwrap().task_manager_cb
    }

    #[inline]
    pub fn lua_manager(&self) -> &LuaManager {
        &self.0.components.as_ref().unwrap().lua_manager
    }

    // #[inline]
    // pub fn schedule_manager(&self) -> &ScheduleManager {
    //     &self.0.components.as_ref().unwrap().schedule_manager
    // }

    #[inline]
    pub fn thing_components(&self) -> &ThingCollect {
        &self.0.components.as_ref().unwrap().things_components
    }
}

unsafe impl Sync for Process {}
unsafe impl Send for Process {}

impl Process {
    async fn subscribe_message(&self) -> NearResult<()> {
        {
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_SEARCH_PUB.topic(),
                    move || Ok(SearchRoutine::open(arc_self.clone()))
                )?;
        }

        {
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_TASK_RESULT_PUB.topic(),
                    move || Ok(GetTaskResultRoutine::new(arc_self.clone()))
                )?;
        }

        {
            // add thing
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_ADD_THING_PUB.topic(),
                    move || Ok(AddThingTaskRoutine::new(arc_self.clone()))
                )?;
        }

        {
            // remove thing
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_REMOVE_THING_PUB.topic(),
                    move || Ok(CrudThingTaskRoutine::new(arc_self.clone(), crate::tasks::TaskModule::RemoveThing))
                )?;
        }

        {
            // pair thing
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_PAIR_THING_PUB.topic(),
                    move || Ok(CrudThingTaskRoutine::new(arc_self.clone(), crate::tasks::TaskModule::PairThing))
                )?;
        }

        {
            // remove pair thing
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_REMOVE_PAIR_THING_PUB.topic(),
                    move || Ok(CrudThingTaskRoutine::new(arc_self.clone(), crate::tasks::TaskModule::RemovePairThing))
                )?;
        }

        {
            // query thing
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_QUERY_THING_PUB.topic(),
                    move || Ok(CrudThingTaskRoutine::new(arc_self.clone(), crate::tasks::TaskModule::QueryThing))
                )?;
        }

        {
            // control thing
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_CONTROL_THING_PUB.topic(),
                    move || Ok(ControlThingTaskRoutine::new(arc_self.clone()))
                )?;
        }

        {
            // query all thing
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_QUERY_ALL_THING_PUB.topic(),
                    move || Ok(QueryAllThingTaskRoutine::new(arc_self.clone()))
                )?;
        }

        {
            // add schedule
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_SCHEDULE_ADD_PUB.topic(),
                    move || Ok(AddScheduleRoutine::new(arc_self.clone()))
                )?;
        }

        {
            // remove schedule
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_SCHEDULE_REMOVE_PUB.topic(),
                    move || Ok(RemoveScheduleRoutine::new(arc_self.clone()))
                )?;
        }

        {
            // execute schedule
            let arc_self = self.clone();

            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_private_topic(
                    NEAR_THING_SERVICE_SCHEDULE_EXECUTE_PUB.topic(),
                    move || Ok(ExecuteScheduleRoutine::new(arc_self.clone()))
                )?;
        }

        Ok(())
    }

    async fn sync_thing(&self) -> NearResult<Vec<ThingObject>> {
        let routine = 
            RoutineTemplate::<Thing_info_list>::call(
                NEAR_THING_STORAGE_THING_QUERY_ALL_PUB.topic().clone(),
                Thing_query_all::default()
            )
            .await
            .map_err(| e | {
                error!("failed call {}", NEAR_THING_STORAGE_THING_QUERY_ALL_PUB.topic());
                e
            })?;

        let thing_ids: Vec<String> = 
        async_std::future::timeout(std::time::Duration::from_secs(5), routine)
            .await
            .map_err(| _e |{
                NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, "timeout")
            })?
            .map(| v | {
                v.things.into_iter()
                    .map(| thing | thing.thing_id)
                    .collect()
            })
            .map_err(| e | {
                error!("{e}");
                e
            })?;

        RoutineTemplate::<Vec<Option<ThingObject>>>::call(
            NEAR_THING_STORAGE_MULITPLE_THINGOBJECT_QUERY_PUB.topic().clone(),
            thing_ids
        )
        .await
        .map_err(| e | {
            error!("failed call {}", NEAR_THING_STORAGE_MULITPLE_THINGOBJECT_QUERY_PUB.topic());
            e
        })?
        .await
        .map(| v | {
            v.into_iter()
                 .filter(| it | {
                    it.is_some()
                 })
                 .map(| it | {
                    it.unwrap()
                 })
                 .collect()
        })

    }

    async fn init_thing_components(&self) -> NearResult<()> {
        let things = loop {
            match self.sync_thing().await {
                Ok(things) => break(Some(things)),
                Err(e) => {
                    match e.errno() {
                        ErrorCode::NEAR_ERROR_TIMEOUT => continue,
                        _ => break(None)
                    }
                }
            }
        };
    
        if let Some(things) = things {
            self.thing_components().add_things(things.into_iter());
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {

    async fn run(&self) -> NearResult<()> {
        trace!("run...");

        // start scanning
        ScanProcessor::get_instance().active(std::time::Duration::ZERO, self.task_manager_cb().clone_as_event()).await?;

        self.subscribe_message().await?;

        self.init_thing_components().await?;

        // self.schedule_manager().start();

        Ok(())
    }

    fn quit(&self) {
        trace!("quiting...");

        async_std::task::block_on(async move {
            // stop scanning
            ScanProcessor::get_instance().wait_and_close().await;
            // stop advertising
            AdvertisingProcessor::get_instance().wait_and_close().await;
        })
    }

}

#[async_trait::async_trait]
impl TaskCbTrait for Process {
    async fn on_taskcb(&self, task_module: TaskModule, data: TaskCbData) {
        match task_module {
            TaskModule::QueryThing => {
                self.thing_components().on_taskcb(task_module, data).await
            }
            TaskModule::Search => unreachable!(),
            _ => { todo!() }
        }
    }
}
