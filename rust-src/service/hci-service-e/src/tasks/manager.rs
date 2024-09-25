
use std::{sync::Arc, time::Duration};

use log::error;
use near_base::queue::Queue;
use near_base::{NearResult, ErrorCode};
use topic_util::types::hci_types::HciTaskId;

use crate::SEQ;
use crate::lua::configure::ConfigureData;
use crate::{hci::advertising::AdvertisingProcessor, TIMES};
use crate::process::Process;
use crate::lua::data::Data;

use super::{TaskData, TaskModule};

#[derive(Clone)]
pub struct Config {
    pub(crate) interval: Duration,
    pub(crate) times: u8,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(3),
            times: 5,
        }
    }
}

struct ManagerImpl {
    process: Process,
    queues: Queue<TaskData>,
    config: Config,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn start(process: Process) -> NearResult<Self> {
        let this = Self(Arc::new(ManagerImpl{
            process,
            queues: Default::default(),
            config: Default::default(),
        }));

        {
            let this_clone = this.clone();
            async_std::task::spawn(async move {
                this_clone.process().await;
            });
        }

        Ok(this)
    }

    pub(in self) async fn process(&self) {

        loop {
            if let Some(data) = self.0.queues.wait_and_take(std::time::Duration::from_millis(1)).await {
                self.advertising(
                    data, 
                    self.0.config.interval, 
                    self.0.config.times
                )
                .await;
                // let data = Data::from(task_data.params);
                // data.set(SEQ.to_owned(), ConfigureData::get_instace().gen_serial_num().to_string());

                // for t in 0..times {
                //     let data_clone = data.clone();
                //     data_clone.set(TIMES.to_owned(), t.to_string());
                //     // get advertising data for search task
                //     let advertising_data =
                //         match arc_self.0
                //                 .process
                //                 .lua_manager()
                //                 .call(&task_data.module_id,
                //                         task_data.task_module.to_str(),
                //                         data_clone)
                //                 .await {
                //             Ok(v) => v,
                //             Err(e) => {
                //                 error!("failed get advertising data with err: {e}");
                //                 continue;
                //             }
                //         };

                //     if let Err(e) = AdvertisingProcessor::get_instance().add_data(advertising_data) {
                //         error!("failed insert hci process list with err: {e}");
                //     }

                //     let _ = async_std::future::timeout(interval, async_std::future::pending::<()>()).await;
                // }
            }
        }

    }
}

impl Manager {
    pub async fn add_task(&self, task_data: TaskData) -> NearResult<HciTaskId> {
        match &task_data.task_module {
            TaskModule::Search => self.add_search_task(task_data).await,
            _ => self.add_thing_task(task_data).await,
        }
    }
}

impl Manager {
    async fn advertising(&self, task_data: TaskData, interval: Duration, times: u8) {
        let data = Data::from(task_data.params);
        data.set(SEQ.to_owned(), ConfigureData::get_instace().gen_serial_num().to_string());

        for t in 0..times {
            let data_clone = data.clone();
            data_clone.set(TIMES.to_owned(), t.to_string());
            // get advertising data for search task
            let advertising_data =
                match self.0
                        .process
                        .lua_manager()
                        .call(&task_data.module_id,
                              task_data.task_module.to_str(),
                              data_clone)
                        .await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("failed get advertising data with err: {e}");
                        continue;
                    }
                };

            if let Err(e) = AdvertisingProcessor::get_instance().add_data(advertising_data) {
                error!("failed insert hci process list with err: {e}");
            }

            let _ = async_std::future::timeout(interval, async_std::future::pending::<()>()).await;
        }

    }

    // fn scheduler_advertising(&self, task_data: TaskData, interval: Duration, times: u8) {
    //     let arc_self = self.clone();

    //     async_std::task::spawn(async move {
    //         let data = Data::from(task_data.params);
    //         data.set(SEQ.to_owned(), ConfigureData::get_instace().gen_serial_num().to_string());

    //         for t in 0..times {
    //             let data_clone = data.clone();
    //             data_clone.set(TIMES.to_owned(), t.to_string());
    //             // get advertising data for search task
    //             let advertising_data =
    //                 match arc_self.0
    //                         .process
    //                         .lua_manager()
    //                         .call(&task_data.module_id,
    //                                 task_data.task_module.to_str(),
    //                                 data_clone)
    //                         .await {
    //                     Ok(v) => v,
    //                     Err(e) => {
    //                         error!("failed get advertising data with err: {e}");
    //                         continue;
    //                     }
    //                 };

    //             if let Err(e) = AdvertisingProcessor::get_instance().add_data(advertising_data) {
    //                 error!("failed insert hci process list with err: {e}");
    //             }

    //             let _ = async_std::future::timeout(interval, async_std::future::pending::<()>()).await;
    //         }
    //     });

    // }

    async fn add_search_task(&self, task_data: TaskData) -> NearResult<HciTaskId> {
        let task_id =
            // startup advertising process
            match AdvertisingProcessor::get_instance()
                    .active()
                    .await {
                Ok(()) => Ok(task_data.task_module.into_value()),
                Err(e) => {
                    match e.errno() {
                        ErrorCode::NEAR_ERROR_STARTUP => Ok(task_data.task_module.into_value()),
                        _ => Err(e)
                    }
                }
            }?;

        self.0.queues.push(task_data, Some(near_base::queue::PushMethod::PushHead));
        // self.scheduler_advertising(task_data, self.0.process.config().task_config.interval, self.0.process.config().task_config.times);

        Ok(task_id)
    }

    async fn add_thing_task(&self, task_data: TaskData) -> NearResult<HciTaskId> {
        // startup advertising process
        let task_id =
            match AdvertisingProcessor::get_instance()
                    .active()
                    .await {
                Ok(()) => Ok(task_data.task_module.into_value()),
                Err(e) => {
                    match e.errno() {
                        ErrorCode::NEAR_ERROR_STARTUP => Ok(task_data.task_module.into_value()),
                        _ => Err(e)
                    }
                }
            }?;

        match &task_data.task_module {
            TaskModule::QueryThing => self.0.queues.push(task_data, None),
            _ => self.0.queues.push(task_data, Some(near_base::queue::PushMethod::PushHead)),
        }
        // self.scheduler_advertising(task_data, self.0.process.config().ctrl_task_config.interval, self.0.process.config().ctrl_task_config.times);

        Ok(task_id)
    }
}


