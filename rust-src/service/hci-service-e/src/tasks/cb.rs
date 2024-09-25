
use std::{sync::Arc, str::FromStr};

use bluex::management::scaner::ScanResult;
use log::{error, debug};

use crate::{hci::scanning::ScanProcessorEventTrait, process::Process, tasks::TaskCbData};

use super::{TaskModule, result::search_result::SeachEventResult, TaskCbTrait};

struct TaskManagerCbImpl {
    process: Process,
    cb: Box<dyn TaskCbTrait>,
}

#[derive(Clone)]
pub struct TaskManagerCb(Arc<TaskManagerCbImpl>);

impl TaskManagerCb {
    pub fn new(process: Process, cb: Box<dyn TaskCbTrait>) -> Self {
        let ret = Self(Arc::new(TaskManagerCbImpl{
            process,
            cb,
        }));

        ret
    }

    pub(in self) fn process(&self) -> &Process {
        &self.0.process
    }
}

#[async_trait::async_trait]
impl ScanProcessorEventTrait for TaskManagerCb {
    fn clone_as_event(&self) -> Box<dyn ScanProcessorEventTrait> {
        Box::new(self.clone())
    }

    async fn scan_event(&self, result: Vec<ScanResult>) {
        let mut fut = vec![];
        for r in result {
            fut.push(self.analyze_data(r));
        }

        let _ = futures::future::join_all(fut).await;
    }

}

impl TaskManagerCb {
    async fn analyze_data(&self, r: ScanResult) {
        let (mac, data) = (r.addr, r.data);

        debug!("analyze_data: mac:{} data:{}", mac.to_string(), hex::encode_upper(data.as_slice()));

        // // merge thing
        // let input_data = 
        //     if let Ok(thing) = self.process().thing_components().get_thing_by_mac(mac.clone().bytes()) {
        //         match thing.status() {
        //             ThingStatus::Disable => { info!("mac: {} has been disabled, ignore.", mac.to_string()); return; },
        //             _ => Some(thing.thing().body().content().user_data().clone()),
        //         }
        //     } else {
        //         None
        //     }
        //     .unwrap_or(Default::default());

        // analyze in built-in
        let output = 
            match self.process()
                      .lua_manager()
                      .analyze_data(data, Default::default())
                      .await {
            Ok(output) => output,
            Err(e) => {
                let error_string = format!("failed analyze-data with err: {e}");
                error!("{error_string}");
                return;
            }
        };

        debug!("successfully analyze thing data: {}, thing-data:{}", mac.to_string(), output);
        let cmd = output.get_cmd();
        let mac = output.get_mac().unwrap_or(mac);
        if let Ok(task_module) = TaskModule::from_str(&cmd) {
            match task_module {
                TaskModule::Search => SeachEventResult::get_instance().on_taskcb(task_module, TaskCbData::from((mac, output))).await,
                _ => self.0.cb.on_taskcb(task_module, TaskCbData::from((mac, output))).await,
            }
        } else {
            error!("Unidentified {{{cmd}}} TaskModule");
        }
    }

}
