
use log::{trace, error};

use near_base::{thing::ThingObject, NearResult};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, };

use crate::process::Process;

use super::p::CheckAndGetThingObject;

pub struct QueryMultipleThingObjectRoutine {
    process: Process,
}

impl QueryMultipleThingObjectRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(Self{
            process,
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryMultipleThingObjectRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("QueryMultipleThingObjectRoutine::on_routine: header_meta={header_meta}.");

        let r = 
            match protos::RawObjectHelper::decode::<Vec<String>>(req) {
                Ok(data) => {
                match data {
                    protos::DataContent::Content(o) => { DataContent::Content(o) },
                    protos::DataContent::Error(_) => unreachable!()
                }
                }
                Err(e) => {
                    let error_string = format!("failed decode message with err={e}");
                    log::error!("{error_string} sequence={}", header_meta.sequence());
                    DataContent::Error(e)
                }
            };

        let r: DataContent<Vec<Option<ThingObject>>> = match r {
            DataContent::Content(thing_ids) => self.on_routine(header_meta, thing_ids).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryMultipleThingObjectRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, thing_ids: Vec<String>) -> NearResult<Vec<Option<ThingObject>>> {
        let mut r = vec![];

        for thing_id in thing_ids {
            r.push(CheckAndGetThingObject::call(self.process(), &self.process().config().thing_data_path, &thing_id)
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })
                    .ok());
        }

        Ok(r)
    }
}


