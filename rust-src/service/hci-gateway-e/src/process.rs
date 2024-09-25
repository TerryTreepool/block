
use std::{sync::Arc, path::PathBuf};

use log::trace;

use near_base::NearResult;

use common::{RuntimeProcessTrait, RuntimeStack};
use topic_util::topics::hci_gateway::*;

use crate::routines::schedule::add::AddScheduleRoutine;
use crate::routines::schedule::remove::RemoveScheduleRoutine;
use crate::routines::schedule::update::UpdateScheduleRoutine;
use crate::routines::schedule::update_relations::UpdateScheduleRelationsRoutine;
use crate::routines::things::add_thing::AddThingRoutine;
use crate::routines::things::crud_thing::CrudThingRoutine;
use crate::routines::things::ctrl_thing::CtrlThingRoutine;
use crate::routines::things::search::SearchRoutine;
use crate::routines::things::search_result::SearchResultRoutine;

#[derive(Clone)]
struct Config {
    #[allow(unused)]
    work_path: PathBuf,
    #[allow(unused)]
    thing_cache_path: PathBuf,
}

struct ProcessComponents {
}

struct ProcessImpl {
    #[allow(unused)]
    service_name: String,
    #[allow(unused)]
    config: Config,

    components: Option<ProcessComponents>,
}

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

impl Process {
    pub fn new(service_name: String,) -> Self {
        let work_path = near_core::get_service_path(service_name.as_str());

        let ret = Self(Arc::new(ProcessImpl {
            service_name,
            config: Config {
                thing_cache_path: {
                    let thing_cache_path = work_path.join("things");
                    let _ = std::fs::create_dir_all(thing_cache_path.as_path());
                    thing_cache_path
                },
                work_path: work_path,
            },
            components: None
        }));

        let components = ProcessComponents {
        };

        {
            let mut_ret = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ProcessImpl) };
            mut_ret.components = Some(components);
        }

        ret
    }

}

impl Process {
    pub(in self) async fn subscribe_things_topic(&self) -> NearResult<()> {
        {
            let arc_self = self.clone();
            // search device
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_GATEWAY_SEARCH_PUB.topic(),
                    move || Ok(SearchRoutine::new(arc_self.clone()))
                )?;
        }

        {
            let arc_self = self.clone();
            // get search result
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_GATEWAY_SEARCH_RESULT_PUB.topic(),
                    move || Ok(SearchResultRoutine::new(arc_self.clone()))
                )?;
        }

        {
            let arc_self = self.clone();
            // add
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_GATEWAY_ADD_THING_PUB.topic(),
                    move || Ok(AddThingRoutine::new(arc_self.clone()))
                )?;
        }

        {
            let arc_self = self.clone();
            // crud thing
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_GATEWAY_CRUD_THING_PUB.topic(),
                    move || Ok(CrudThingRoutine::new(arc_self.clone()))
                )?;
        }

        {
            let arc_self = self.clone();
            // ctrl thing
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_GATEWAY_CTRL_THING_PUB.topic(),
                    move || Ok(CtrlThingRoutine::new(arc_self.clone()))
                )?;
        }

        Ok(())
    }

    pub async fn subscribe_schedule_topic(&self) -> NearResult<()> {
        {
            let arc_self = self.clone();
            // add schedule
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_GATEWAY_SCHEDULE_ADD_PUB.topic(),
                    move || Ok(AddScheduleRoutine::new(arc_self.clone()))
                )?;
        }

        {
            let arc_self = self.clone();
            // remove schedule
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_GATEWAY_SCHEDULE_REMOVE_PUB.topic(),
                    move || Ok(RemoveScheduleRoutine::new(arc_self.clone()))
                )?;
        }

        {
            let arc_self = self.clone();
            // update schedule
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_GATEWAY_SCHEDULE_UPDATE_PUB.topic(),
                    move || Ok(UpdateScheduleRoutine::new(arc_self.clone()))
                )?;
        }

        {
            let arc_self = self.clone();
            // update relations schedule
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_GATEWAY_SCHEDULE_UPDATE_RELATIONS_PUB.topic(),
                    move || Ok(UpdateScheduleRelationsRoutine::new(arc_self.clone()))
                )?;
        }

        Ok(())
    }

}

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {

    async fn run(&self) -> NearResult<()> {
        trace!("run enter");

        self.subscribe_things_topic().await?;
        self.subscribe_schedule_topic().await?;

        Ok(())
    }

    fn quit(&self) {
        trace!("quiting...");
    }
}

