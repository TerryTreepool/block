
use common::RoutineTemplate;
use log::{trace, error};

use near_base::{NearResult, builder_codec_macro::Empty};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, hci::schedule::{Schedule_add, Schedule_info, Schedule_relation_list, Schedule_mode}};
use topic_util::topics::{hci_storage::NEAR_THING_STORAGE_SCHEDULE_ADD_PUB, hci_schedule::NEAR_THING_SCHEDULE_ADD_PUB, hci_service::NEAR_THING_SERVICE_SCHEDULE_ADD_PUB};

use crate::process::Process;

pub struct AddScheduleRoutine {
    _process: Process,
}

impl AddScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddScheduleRoutine{
            _process: process
        }))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("AddScheduleRoutine: header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(Schedule_add, req, o, o, { header_meta.sequence() });

        let r: DataContent<Schedule_info> = match r {
            DataContent::Content(schedule_data) => 
                self.on_routine(header_meta, schedule_data).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, schedule_data: Schedule_add) -> NearResult<Schedule_info> {

        // check right
        // todo

        let schedule_info = 
            RoutineTemplate::<Schedule_info>::call_with_headermeta(
                header_meta,
                NEAR_THING_STORAGE_SCHEDULE_ADD_PUB.topic().clone(),
                schedule_data
            )
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        match schedule_info.mode() {
            Schedule_mode::None => Ok(schedule_info),
            _ => {
                let _ =
                    RoutineTemplate::<Empty>::call_with_headermeta(
                        header_meta,
                        NEAR_THING_SERVICE_SCHEDULE_ADD_PUB.topic().clone(),
                        (
                                    schedule_info.schedule_id().to_owned(), 
                                    Schedule_relation_list {
                                        thing_relation: schedule_info.thing_relation().to_vec(),
                                        ..Default::default()
                                    }
                                )
                    )
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })?
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })?;

                let _ = 
                    RoutineTemplate::<Empty>::call_with_headermeta(
                        header_meta, 
                        NEAR_THING_SCHEDULE_ADD_PUB.topic().clone(),
                        schedule_info.clone()
                    )
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })?
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })?;

                Ok(schedule_info)
            }
        }
    }

}