
use log::{trace, error};

use near_base::{thing::ThingObject, NearResult};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, };

use crate::process::Process;

pub struct QueryThingObjectRoutine {
    process: Process,
}

impl QueryThingObjectRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(Self{
            process,
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryThingObjectRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("QueryThingObjectRoutine::on_routine: header_meta={header_meta}.");

        let r = try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<ThingObject> = match r {
            DataContent::Content(thing_id) => {
                self.on_routine(header_meta, thing_id).await.into()
            }
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryThingObjectRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, thing_id: String) -> NearResult<ThingObject> {

        let (thing, thing_object) = 
            self.process
                .thing_storage()
                .load_with_prefix(&thing_id)
                .await
                .map(| thing | {
                    thing.split()
                })
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        // check brand
        self.process
            .brand_storage()
            .load_with_prefix(&thing.brand_id)
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        Ok(thing_object)

    }
}
