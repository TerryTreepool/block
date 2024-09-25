
use log::{trace, error};

use near_base::{NearResult, builder_codec_macro::Empty};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, hci::product::Product_info, try_decode_raw_object, try_encode_raw_object};

use crate::process::Process;

pub struct RemoveProductRoutine {
    process: Process,
}

impl RemoveProductRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(RemoveProductRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RemoveProductRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RemoveProductRoutine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Product_info, req, o, { (o.take_parent_product_id(), o.take_product_id()) }, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content((parent_product_id, product_id)) => {
                self.on_routine(header_meta, parent_product_id, product_id).await.into()
            }
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl RemoveProductRoutine {

    async fn on_routine(&self, header_meta: &HeaderMeta, parent_product_id: String, product_id: String) -> NearResult<Empty> {

        let _ = 
            if !parent_product_id.is_empty() {
                // delete child products from parent product
                let mut product = 
                    self.process
                        .product_storage()
                        .load_with_prefix(&parent_product_id)
                        .await
                        .map_err(| e | {
                            error!("Not found [{}] root product, sequence: {}.", parent_product_id, header_meta.sequence());
                            e
                        })?;

                product.remove_child(&product_id);

                self.process
                    .product_storage()
                    .update(&product)
                    .await
                    .map_err(| e | {
                        error!("failed remove [{}] product with err: {e}, sequence: {}", parent_product_id, header_meta.sequence());
                        e
                    })?;
            } else {
                // delete root product
                self.process
                    .product_storage()
                    .delete_with_prefix(&product_id)
                    .await
                    .map_err(| e | {
                        error!("failed remove [{}] product with err: {e}, sequence: {}", product_id, header_meta.sequence());
                        e
                    })?;
            };

        Ok(Empty)

    }
}