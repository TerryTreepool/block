
use log::{trace, error};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, thing_group::Thing_group_list};

use crate::process::Process;

pub struct QueryAllGroupRoutine {
    process: Process,
}

impl QueryAllGroupRoutine {
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
impl Routine<RawObjectGuard, RawObjectGuard> for QueryAllGroupRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, _req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("query all group routine: header_meta={header_meta}.");

        let r: DataContent<Thing_group_list> = self.on_routine(header_meta).await.into();

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryAllGroupRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta) -> NearResult<Thing_group_list> {
        let groups = 
        crate::public::group::get_group_list(self.process().db_helper())
            .await
            .map_err(| e | {
                error!("{e}, sequenc: {}", header_meta.sequence());
                e
            })?;

        Ok(Thing_group_list { groups, ..Default::default() })
    }
}
