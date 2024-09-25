
use log::{trace, error, info};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, thing_group::{Thing_group_query, Thing_group_info}};

use crate::process::Process;

pub struct QueryGroupRoutine {
    process: Process,
}

impl QueryGroupRoutine {
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
impl Routine<RawObjectGuard, RawObjectGuard> for QueryGroupRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("query group routine: header_meta={header_meta}.");

        let r = try_decode_raw_object!(Thing_group_query, req, o, { o.take_group_id()}, { header_meta.sequence() });

        let r: DataContent<Thing_group_info> = match r {
            DataContent::Content(group_id) => self.on_routine(header_meta, group_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryGroupRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, group_id: String) -> NearResult<Thing_group_info> {
        crate::public::group::get_group(self.process().db_helper(), 
                                        group_id.as_str())
            .await
            .map(| group | {
                info!("group: {group}");
                group
            })
            .map_err(| e |{
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })
    }
}
