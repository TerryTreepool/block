
use std::sync::Arc;

use common::RoutineTemplate;
use log::{trace, error};

use near_base::{NearResult, builder_codec_macro::Empty};
use near_transport::{Routine, HeaderMeta, RoutineWrap, RoutineEventTrait, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};
use topic_util::{topics::hci_service::NEAR_THING_SERVICE_SCHEDULE_EXECUTE_PUB, types::thing_data::{ThingData, ThingId}};

use crate::process::Process;

pub struct ExecuteSchuleRoutine {
    #[allow(unused)]
    process: Process,
}

impl ExecuteSchuleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(ExecuteSchuleRoutine{
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
impl Routine<RawObjectGuard, RawObjectGuard> for ExecuteSchuleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("ExecuteSchuleRoutine::on_routine header_meta={header_meta}");

        let r = try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(c) => {
                self.on_routine(header_meta, c)
                    .await
                    .map(|_| Empty)
                    .into()
            },
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl ExecuteSchuleRoutine {
    pub(in self) async fn on_routine(&self, header_meta: &HeaderMeta, schedule_id: String) -> NearResult<()> {
        let header_meta_ptr = Arc::new(header_meta.clone());
        let schedule_id_clone = schedule_id.clone();

        self.process()
            .schedule_manager()
            .execute(&schedule_id, move | thing_dataes: Vec<(ThingId, ThingData)> | {
                let header_meta = header_meta_ptr.clone();
                let schedule_id = schedule_id_clone.clone();

                async move {
                    RoutineTemplate::<Empty>::call_with_headermeta(
                        header_meta.as_ref(),
                        NEAR_THING_SERVICE_SCHEDULE_EXECUTE_PUB.topic().clone(), 
                        (
                                    schedule_id,
                                    {
                                        let v: Vec<String> = 
                                            thing_dataes.into_iter()
                                                .map(| (thing_id, _) | {
                                                    thing_id.to_string()
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
                    .map(| _ | ())
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })

                }
            })
            .await
            .map_err(| e | {
                error!("{e}, sequecne: {}", header_meta.sequence());
                e
            })

        /*
        self.process()
            .schedule_manager()
            .execute(schedule_id, move | thing_dataes: Vec<(ThingId, ThingData)> | {

                let header_meta = header_meta_ptr.clone();

                async move {

                    let mut thing_array = vec![];
                    for (thing_id, mut thing_data) in thing_dataes {
                        thing_array.push((
                            thing_id.to_string(),
                            thing_data.take_map()
                        ))
                    }

                    RoutineTemplate::<HciTaskId>::call_with_headermeta(
                            header_meta.as_ref(),
                            NEAR_THING_SERVICE_CONTROL_THING_PUB.topic().clone(), 
                            thing_array
                        )
                        .map_err(| e | {
                            error!("{e}, sequence: {}", header_meta.sequence());
                            e
                        })?
                        .await
                        .map(| _ | ())
                        .map_err(| e | {
                            error!("{e}, sequence: {}", header_meta.sequence());
                            e
                        })
                }
            })
            .await
            .map_err(| e | {
                error!("{e}, sequecne: {}", header_meta.sequence());
                e
            })
         */
    
    }

}

