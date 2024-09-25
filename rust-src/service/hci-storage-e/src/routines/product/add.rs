
use log::{trace, error};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{hci::product::{Product_add, Product_info}, DataContent, try_decode_raw_object, try_encode_raw_object};

use crate::{process::Process, caches::product::ProductItem};

pub struct AddProductRoutine {
    process: Process,
}

impl AddProductRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddProductRoutine{
            process
        }))
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

        let (product, create_new) = {
            let parent_product_id = product.parent_product_id.as_str().trim();
            if !parent_product_id.is_empty() {
                let mut parent_product = 
                    self.process
                        .product_storage()
                        .load_with_prefix(parent_product_id)
                        .await
                        .map_err(| e | {
                            error!("Not found [{}] product, sequence: {}.", product.parent_product_id(), header_meta.sequence());
                            e
                        })?;

                parent_product.insert_child(product.take_product_name());

                (parent_product, false)
            } else {
                (ProductItem::create_new(product.take_product_name())
                    .map_err(| e | {
                        error!("failed create product with err: {e}, sequence: {}.", header_meta.sequence());
                        e
                    })?,
                 true)
            }
        };

        if create_new {
            self.process
                .product_storage()
                .create_new(&product)
        } else {
            self.process
                .product_storage()
                .update(&product)
        }
        .await
        .map_err(| e | {
            error!("faild {} [{}] with err: {e}, sequence: {}.", if create_new { "create new" } else { "update" }, product.product_id(), header_meta.sequence());
            e
        })?;

        Ok(product.take())
    }
}