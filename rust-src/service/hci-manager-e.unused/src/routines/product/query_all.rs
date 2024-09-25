
use log::{trace, error};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{product::{Product_info_list, Product_info}, DataContent, try_encode_raw_object};

use crate::process::Process;

pub struct QueryAllProductRoutine {
    process: Process
}

impl QueryAllProductRoutine {
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
impl Routine<RawObjectGuard, RawObjectGuard> for QueryAllProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, _req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("query product routine: header_meta: {header_meta}");

        // let r = try_decode_raw_object!(Product_query_all, req, o, o, { header_meta.sequence() });

        let r: DataContent<Product_info_list> = self.on_routine(header_meta).await.into();

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryAllProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta) -> NearResult<Product_info_list> {
        Ok(Product_info_list {
            products:   self.process()
                            .db_helper()
                            .query_all::<Product_info>(crate::p::GET_ALL_PRODUCT.0)
                            .await
                            .map_err(| e | {
                                error!("{e}, sequence: {}", header_meta.sequence());
                                e
                            })?,
            ..Default::default()
        })
    }
}