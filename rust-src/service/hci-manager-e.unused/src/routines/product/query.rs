
use log::{trace, info};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, product::{Product_query, Product_info}, try_decode_raw_object, try_encode_raw_object};

use crate::process::Process;

pub struct QueryProductRoutine {
    process: Process,
}

impl QueryProductRoutine {
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
impl Routine<RawObjectGuard, RawObjectGuard> for QueryProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("query a product routine: header_meta={header_meta}.");

        let r = try_decode_raw_object!(Product_query, req, o, { o.take_product_id() }, { header_meta.sequence() });

        let r: DataContent<Product_info> = match r {
            DataContent::Content(product_id) => {
                self.on_routine(header_meta, product_id).await.into()
            }
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, product_id: String) -> NearResult<Product_info> {

        crate::public::product::get_product(self.process().db_helper(), 
                                            product_id.as_str())
            .await
            .map_err(| e | {
                info!("{e}, sequence = {}", header_meta.sequence());
                e
            })

    }
}
