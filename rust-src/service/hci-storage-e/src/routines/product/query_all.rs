
use log::{trace, error};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{hci::product::Product_info_list, DataContent, try_encode_raw_object};

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

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryAllProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, _req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("query product routine: header_meta: {header_meta}");

        let r: DataContent<Product_info_list> = self.on_routine(header_meta).await.into();

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryAllProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta) -> NearResult<Product_info_list> {

        let r = 
            self.process
                .product_storage()
                .load()
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        Ok(
            Product_info_list {
                products: {
                    r.into_iter()
                    .map(| product | {
                        product.take()
                    })
                    .collect()    
                },
                ..Default::default()
            }
        )
    }
}