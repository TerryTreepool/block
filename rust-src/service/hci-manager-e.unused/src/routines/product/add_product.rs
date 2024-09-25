
use log::{trace, error, info};

use near_base::{ErrorCode, NearError, NearResult, };
use near_core::time_utils::native_now;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{product::{Product_add, Product_info}, DataContent, try_decode_raw_object, try_encode_raw_object};
use topic_util::types::brand_types::Status;

use crate::{process::Process, public::CheckTrait};
use super::ProductIdBuilder;

pub struct AddProductRoutine {
    process: Process,
}

impl AddProductRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddProductRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("AddProductRoutine::on_routine: header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(Product_add, req, o, { o.take_product() }, { header_meta.sequence() });

        let r: DataContent<Product_info> = match r {
            DataContent::Content(product) => self.on_routine(header_meta, product).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, mut product: Product_info) -> NearResult<Product_info> {

        let product_name = product.product_name().trim();
        // 1. check product name
        if product_name.is_empty() {
            error!("product name is empty, sequence = {}", header_meta.sequence());
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "product name is empty"))
        } else {
            Ok(())
        }?;

        // check parent product
        if !product.parent_product_id.is_empty() {
            crate::public::product::get_product(self.process().db_helper(), product.parent_product_id())
                .await
                .map_err(| e |{
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?
                .check_status()?;
        }

        // 3. build product id and time
        let new_product = {
            let now = native_now().format("%Y-%m-%d %H:%M:%S").to_string();
            Product_info {
                product_id: ProductIdBuilder {
                    parent_product_id: product.parent_product_id(),
                    product_name: product.product_name(),
                }.build(),
                product_name: product.take_product_name(),
                parent_product_id: product.take_parent_product_id(),
                begin_time: now.clone(),
                update_time: now,
                status: Status::Eanbled.into(),
                ..Default::default()
            }
        };

        self.process()
            .db_helper()
            .execute_with_param(crate::p::ADD_PRODUCT.0, &new_product)
            .await
            .map(| _ | {
                info!("success add {} brand info, sequence = {}", new_product.product_id(), header_meta.sequence());
                new_product
            })
            .map_err(| e | {
                let error_string = format!("failed add product with err = {e}");
                error!("{error_string}, sequence = {}", header_meta.sequence());
                e
            })

    }
}