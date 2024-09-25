
use log::{trace, error};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{brand::{Brand_info_list, Brand_info}, try_encode_raw_object, DataContent};

use crate::process::Process;

pub struct QueryAllBrandRoutine {
    process: Process
}

impl QueryAllBrandRoutine {
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
impl Routine<RawObjectGuard, RawObjectGuard> for QueryAllBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, _req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("query brand routine: header_meta: {header_meta}");

        let r: DataContent<Brand_info_list> = self.on_routine(header_meta).await.into();

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryAllBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta) -> NearResult<Brand_info_list> {
        Ok(Brand_info_list {
            brands: self.process()
            .db_helper()
            .query_all::<Brand_info>(crate::p::GET_ALL_BRAND.0)
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?,
            ..Default::default()
        })
    }
}
