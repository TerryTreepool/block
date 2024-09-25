
use log::{trace, error};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, hci::thing::Thing_info, };

use crate::process::Process;

pub struct UpdateThingRoutine {
    process: Process,
}

impl UpdateThingRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(UpdateThingRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for UpdateThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("UpdateThingRoutine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Thing_info, req, o, o, { header_meta.sequence() });

        let r: DataContent<Thing_info> = match r {
            DataContent::Content(thing) => self.on_routine(header_meta, thing).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl UpdateThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, mut new_thing: Thing_info) -> NearResult<Thing_info> {

        let mut thing = 
            self.process
                .thing_storage()
                .load_with_prefix(new_thing.thing_id())
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        thing.set_thing_name(new_thing.take_thing_name());

        self.process
            .thing_storage()
            .update(&thing)
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        let (thing, _) = thing.split();

        Ok(thing)
    }
}
