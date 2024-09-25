
use std::sync::Arc;

use log::debug;
use near_base::{NearResult, thing::ThingObject, ObjectId, now};

use crate::{process::Process, tasks::{TaskData, TaskModule}, cache::ThingStatus};

use super::ScheduleTrait;

struct ManagerImpl {
    query_schedule: QuerySchedule,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new(process: Process) -> Self {
        let ret = Self(Arc::new(ManagerImpl{
            query_schedule: QuerySchedule::new(process.clone()),
        }));

        ret
    }

    pub fn start(&self) {
        let arc_self = self.clone();

        async_std::task::spawn(async move {
            arc_self.0.query_schedule.on_schedule().await;
        });
    }

}

struct QuerySchedule {
    process: Process,
}

impl QuerySchedule {

    pub fn new(process: Process) -> Self {
        Self {
            process,
        }
    }

}

#[async_trait::async_trait]
impl ScheduleTrait for QuerySchedule {
    async fn init_schedule(&self) -> NearResult<()> {
        Ok(())
    }

    fn add_schedule(&self, thing: ThingObject) -> NearResult<()> {
        self.process.thing_components().add_things(vec![thing].into_iter());
        Ok(())
    }

    fn remove_schedule(&self, thing_id: &ObjectId) -> NearResult<()> {
        self.process.thing_components().remove_thing(thing_id);
        Ok(())
    }

    async fn on_schedule(&self) {
        let interval = self.process.config().schedule_config.query_schedule_config.interval;
        let timeout_response = self.process.config().schedule_config.query_schedule_config.timeout_response.as_micros() as u64;

        loop {
            let now = now();
            let things = self.process.thing_components().get_all_thing();

            // check status
            {
                let timeout_things: Vec<&ObjectId> = 
                things.iter()
                    .filter(| it | {
                        match it.status() {
                            ThingStatus::Online(last_updated, _) => {
                                (now - last_updated) > timeout_response
                            }
                            _ => true
                        }
                    })
                    .map(| it | {
                        it.thing().object_id()
                    })
                    .collect();

                self.process.thing_components().offline(timeout_things.into_iter());
            }

            for thing in things.iter() {
                let _ = self.process
                    .task_manager()
                    .add_task(TaskData::from((
                        TaskModule::QueryThing,
                        thing.thing()
                    )))
                    .await;

            let _ = async_std::future::timeout(std::time::Duration::from_millis(3), async_std::future::pending::<()>()).await;
            }
            // let mut fut = vec![];

            // for thing in things.iter() {
            //     fut.push(self.process
            //                 .task_manager()
            //                 .add_task(TaskData::from((
            //                     TaskModule::QueryThing,
            //                     thing.thing()
            //                 ))));
            // }
            // let r = futures::future::join_all(fut).await;

            debug!("execute {} query scheduler.", things.len());

            let _ = async_std::future::timeout(interval, async_std::future::pending::<()>()).await;
        }
    }
}

