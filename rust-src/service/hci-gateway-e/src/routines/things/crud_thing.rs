
use common::RoutineTemplate;
use log::{trace, error};

use near_base::{NearResult, NearError, ErrorCode, builder_codec_macro::Empty};
use near_transport::{Routine, HeaderMeta, RoutineWrap, RoutineEventTrait, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, hci::hci_thing::{Hci_crud_thing, hci_crud_thing::Hci_crud_m}, try_encode_raw_object};
use topic_util::{topics::{hci_service::{NEAR_THING_SERVICE_REMOVE_THING_PUB, 
                                          NEAR_THING_SERVICE_PAIR_THING_PUB, 
                                          NEAR_THING_SERVICE_REMOVE_PAIR_THING_PUB, 
                                          NEAR_THING_SERVICE_QUERY_THING_PUB}, 
                                          hci_storage::NEAR_THING_STORAGE_THING_REMOVE_PUB}, 
                                          types::hci_types::HciTaskId};

use crate::process::Process;

pub struct CrudThingRoutine {
    #[allow(unused)]
    process: Process,
}

impl CrudThingRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(CrudThingRoutine{
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
impl Routine<RawObjectGuard, RawObjectGuard> for CrudThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("CrudThingRoutine::on_routine header_meta={header_meta}");

        let r = try_decode_raw_object!(Hci_crud_thing, req, o, o, { header_meta.sequence() });

        let r: DataContent<HciTaskId> = match r {
            DataContent::Content(c) => {
                self.on_routine(header_meta, c)
                    .await
                    .into()
            },
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl CrudThingRoutine {
    pub(in self) async fn on_routine(&self, header_meta: &HeaderMeta, mut hci_crud_thing: Hci_crud_thing) -> NearResult<HciTaskId> {
        let m = 
            hci_crud_thing.method.enum_value()
                .map_err(| v | {
                    let error_string = format!("unidentified {v} id");
                    error!("{error_string}, sequence: {}", header_meta.sequence());
                    NearError::new(ErrorCode::NEAR_ERROR_TOPIC_EXCEPTION, error_string)
                })?;
        let thing_data = hci_crud_thing.take_data();
        let thing_id = hci_crud_thing.thing_id();

        let r = 
            RoutineTemplate::<HciTaskId>::call_with_headermeta(
                header_meta, 
                {
                    match m {
                        Hci_crud_m::remove => NEAR_THING_SERVICE_REMOVE_THING_PUB.topic(),
                        Hci_crud_m::pair => NEAR_THING_SERVICE_PAIR_THING_PUB.topic(),
                        Hci_crud_m::remove_pair => NEAR_THING_SERVICE_REMOVE_PAIR_THING_PUB.topic(),
                        Hci_crud_m::query => NEAR_THING_SERVICE_QUERY_THING_PUB.topic(),
                    }
                }.clone(), 
                (thing_id.to_owned(), thing_data)
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

        match m {
            Hci_crud_m::remove => {
                RoutineTemplate::<Empty>::call_with_headermeta(
                        header_meta, 
                        NEAR_THING_STORAGE_THING_REMOVE_PUB.topic().clone(),
                        thing_id
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
            }
            _ => {}
        }

        Ok(r)
    }

}

