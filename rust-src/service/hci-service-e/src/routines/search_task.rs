
use std::sync::Arc;

use log::{trace, error, warn};

use near_base::{ErrorCode, NearResult, NearError};
use near_transport::{Routine, HeaderMeta, EventResult, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};
use topic_util::types::hci_types::HciTaskId;

use crate::{tasks::{TaskData, TaskModule, result::search_result::SeachEventResult}, 
            process::Process, };

struct SearchRoutineImpl {
    process: Process,
}

#[derive(Clone)]
pub struct SearchRoutine(Arc<SearchRoutineImpl>);

impl SearchRoutine {
    pub fn open(process: Process) -> Box<dyn RoutineEventTrait> {
        let ret = Self(Arc::new(SearchRoutineImpl{ 
            process
        }));

        RoutineWrap::new(Box::new(ret))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for SearchRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("SearchRoutine::on_routine header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(String, 
                                   req, 
                                   c, 
                                   {
                                        TaskData {
                                            task_module: TaskModule::Search,
                                            module_id: c,
                                            params: Default::default(),
                                        }
                                   },
                                   { header_meta.sequence() });

        let r: DataContent<HciTaskId> = match r {
            DataContent::Content(task_data) => {
                self.add_search_task(header_meta, task_data).await
            }
            DataContent::Error(e) => Err(e)
        }.into();

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl SearchRoutine {
    pub(in self) async fn add_search_task(
        &self, 
        header_meta: &HeaderMeta, 
        task_data: TaskData
    ) -> NearResult<HciTaskId> {

        if let Some(creator) = header_meta.creator.as_ref() {
            let creator = 
                creator.creator.as_ref().ok_or_else(|| {
                    warn!("missing creator. sequence: {}", header_meta.sequence());
                    NearError::new(ErrorCode::NEAR_ERROR_NO_TARGET, "missing creator")
                })?;

            SeachEventResult::get_instance().add_object(creator.clone());
            self.0.process.task_manager().add_task(task_data).await
        } else {
            let error_string = format!("failed search due to missing creator.");
            error!("{error_string}, sequence: {}", header_meta.sequence());
            Err(NearError::new(ErrorCode::NEAR_ERROR_REFUSE, error_string))
        }
    }
}
