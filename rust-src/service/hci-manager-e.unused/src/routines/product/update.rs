
use log::{trace, error, info};

use near_base::NearResult;
use near_core::time_utils::native_now;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, product::{Product_add, Product_info}, try_decode_raw_object, try_encode_raw_object};
use topic_util::types::brand_types::Status;

use crate::{process::Process, public::CheckTrait};

pub struct UpdateProductRoutine {
    process: Process,
}

impl UpdateProductRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(UpdateProductRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for UpdateProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("update product routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Product_add, req, o, { o.take_product() }, { header_meta.sequence() });

        let r: DataContent<Product_info> = match r {
            DataContent::Content(product) => {
                self.on_routine(header_meta, product).await.into()
            }
            DataContent::Error(e) => DataContent::Error(e),
        };

        
        
        

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl UpdateProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, product: Product_info) -> NearResult<Product_info> {
        Status::try_from(product.status())
            .map_err(| e | {
                error!("err = {e}, sequence = {}", header_meta.sequence());
                e
            })?;

        let mut mut_product = 
        crate::public::product::get_product(self.process().db_helper(), product.product_id())
            .await
            .map_err(| e |{
                error!("err = {e}, sequence = {}", header_meta.sequence());
                e
            })?;

            mut_product.check_status()?;

        mut_product.set_status(product.status);
        mut_product.set_update_time(native_now().format("%Y-%m-%d %H:%M:%S").to_string());

        self.process()
            .db_helper()
            .execute_with_param(crate::p::UPDATE_PRODUCT.0, &mut_product)
            .await
            .map(| _ | {
                info!("success update {} product info, sequence = {}", product.product_id(), header_meta.sequence());
                mut_product
            })
            .map_err(| e |{
                let error_string = format!("failed update {} product with err = {e}", product.product_id());
                error!("{error_string}, sequence = {}", header_meta.sequence());
                e
            })
    }
}