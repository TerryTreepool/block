
use common::RoutineTemplate;
use log::{trace, error};

use near_base::{NearResult, builder_codec_macro::Empty};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, hci::schedule::{Schedule_info, Schedule_mode}};
use topic_util::topics::{hci_storage::NEAR_THING_STORAGE_SCHEDULE_REMOVE_PUB, hci_schedule::NEAR_THING_SCHEDULE_REMOVE_PUB, hci_service::NEAR_THING_SERVICE_SCHEDULE_REMOVE_PUB};

use crate::process::Process;

pub struct RemoveScheduleRoutine {
    _process: Process,
}

impl RemoveScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(RemoveScheduleRoutine{
            _process: process
        }))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RemoveScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RemoveScheduleRoutine: header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(schedule_id) => 
                self.on_routine(header_meta, schedule_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl RemoveScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, schedule_id: String) -> NearResult<Empty> {

        let mut schedule_info = 
            RoutineTemplate::<Schedule_info>::call_with_headermeta(
                header_meta, 
                NEAR_THING_STORAGE_SCHEDULE_REMOVE_PUB.topic().clone(),
                schedule_id.as_str()
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
            Schedule_mode::None => Ok(Empty),
            _ => {
                let _ =
                    RoutineTemplate::<Empty>::call_with_headermeta(
                        header_meta,
                        NEAR_THING_SERVICE_SCHEDULE_REMOVE_PUB.topic().clone(),
                        (
                                    schedule_info.schedule_id().to_owned(),
                                    {
                                        let v: Vec<String> = schedule_info.take_thing_relation()
                                                    .into_iter()
                                                    .map(| item | {
                                                        item.thing_id
                                                    })
                                                    .collect();
                                        v
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
                        NEAR_THING_SCHEDULE_REMOVE_PUB.topic().clone(),
                        schedule_id.as_str()
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

                Ok(Empty)
            }
        }
    }

}