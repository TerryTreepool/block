
use log::{trace, error};

use near_base::{thing::ThingObject, NearResult};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object, };

use crate::process::Process;

pub struct QueryMultipleThingObjectRoutine {
    process: Process,
}

impl QueryMultipleThingObjectRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(Self{
            process,
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryMultipleThingObjectRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("QueryMultipleThingObjectRoutine::on_routine: header_meta={header_meta}.");

        let r = try_decode_raw_object!(Vec<String>, req, o, o, { header_meta.sequence() });

        let r: DataContent<Vec<Option<ThingObject>>> = match r {
            DataContent::Content(thing_ids) => self.on_routine(header_meta, thing_ids).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryMultipleThingObjectRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, thing_ids: Vec<String>) -> NearResult<Vec<Option<ThingObject>>> {

        let mut fut = vec![];

        for thing_id in thing_ids.iter() {
            fut.push(
                self.process
                    .thing_storage()
                    .load_with_prefix(thing_id)
            );
        }

        Ok(
            futures::future::join_all(fut).await
                .into_iter()
                .map(| thing | {
                    thing.map(| thing | {
                        let (_, thing_object) = thing.split();
                        thing_object
                    })
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })
                    .ok()
                })
                .collect()
        )

    }
}


