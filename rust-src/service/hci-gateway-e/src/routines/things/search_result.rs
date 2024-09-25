
use base::raw_object::RawObjectGuard;
use common::RoutineTemplate;
use log::{trace, error};
use near_base::NearResult;
use near_transport::{EventResult, RoutineEventTrait, RoutineWrap, Routine, HeaderMeta};
use protos::{DataContent, try_decode_raw_object, hci::hci_thing::{Hci_thing_list, Hci_task_result}, try_encode_raw_object,};
use topic_util::{types::hci_types::HciTaskId, topics::hci_service::NEAR_THING_SERVICE_TASK_RESULT_PUB};

use crate::process::Process;

pub struct SearchResultRoutine {
    #[allow(unused)]
    process: Process,
}

impl SearchResultRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(SearchResultRoutine{
            process
        }))
    }

    #[inline]
    #[allow(unused)]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for SearchResultRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("SearchResultRoutine::on_routine header_meta={header_meta}");

        let r = try_decode_raw_object!(Hci_task_result, req, c, { (c.task_id, c.take_thing_ids()) }, { header_meta.sequence() });

        let r: DataContent<Hci_thing_list> = match r {
            DataContent::Content((task_id, thing_ids)) => self.get_search_result(header_meta, task_id, thing_ids).await,
            DataContent::Error(e) => Err(e)
        }.into();

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl SearchResultRoutine {
    async fn get_search_result(&self, header_meta: &HeaderMeta, task_id: HciTaskId, thing_ids: Vec<String>) -> NearResult<Hci_thing_list> {

        RoutineTemplate::<Hci_thing_list>
            ::call_with_headermeta(header_meta, 
                                   NEAR_THING_SERVICE_TASK_RESULT_PUB.topic().clone(), 
                                   (task_id, thing_ids))
            .await
            .map_err(| e | {
                let error_string = format!("failed call {} with err: {e}", NEAR_THING_SERVICE_TASK_RESULT_PUB.topic());
                error!("{error_string}, sequence: {}", header_meta.sequence());
                e
            })?
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })
        /*
        let device_list = 
            RoutineTemplate::<Device_info_list>::call_with_headermeta(
                header_meta, 
                NEAR_THING_MANAGER_DEVICE_QUERY_ALL_PUB.topic().clone(),
                Device_query_all {
                    thing_id: thing_ids,
                    ..Default::default()
                })
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?
                .take_devices();

        let device_using_list: Vec<(String, String)> = 
            device_list.into_iter()
                .filter(| item | {
                    item.status == Status::Eanbled.into()
                })
                .map(| item | {
                    (item.device_id.clone(), item.mac_address.clone())
                })
                .collect();   

        match RoutineTemplate::<Hci_thing_list>
                ::call_with_headermeta(header_meta,
                                       NEAR_THING_SERVICE_TASK_RESULT_PUB.topic().clone(), 
                                       (task_id, device_using_list)) {
            Ok(routine) => {
                routine.await
            }
            Err(e) => {
                let error_string = format!("failed call {} with err: {e}", NEAR_THING_SERVICE_TASK_RESULT_PUB.topic());
                error!("{error_string}, sequence: {}", header_meta.sequence());
                Err(e)
            }
        }
        */
    }
}
