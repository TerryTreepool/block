
use near_base::{NearResult, NearError, ErrorCode};

use protos::product::Product_info;
use topic_util::types::brand_types::Status;

use super::manager_template::{ItemTrait, UpdateItemTrait, CheckTrait};

#[derive(Clone)]
pub struct ProductItem {
    product: Product_info,
}

impl From<ProductItem> for Product_info {
    fn from(value: ProductItem) -> Self {
        value.product
    }
}

impl From<Product_info> for ProductItem {
    fn from(value: Product_info) -> Self {
        Self {
            product: value,
        }
    }
}

impl ItemTrait for ProductItem {
    fn get_item_id(&self) -> &str {
        self.product.product_id()
    }
}

impl UpdateItemTrait<ProductItem> for ProductItem {
    fn update_item(&mut self, new_item: ProductItem) {
        let mut_product = &mut self.product;
        let product: Product_info = new_item.into();

        debug_assert_eq!(mut_product.product_id(), product.product_id());

        mut_product.set_status(product.status);
        mut_product.set_update_time(product.update_time);
    }
}

impl CheckTrait for ProductItem {
    fn check_status(&self) -> NearResult<()> {
        self.status.try_into()
            .map_or_else(| _ |{
                let error_string = format!("{}'s status is exception, can't use it.", self.product_id());
                Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, error_string))
            },| status | {
                match status {
                    Status::Eanbled => Ok(()),
                    Status::Disabled => {
                        let error_string = format!("{} brand is diabled, cann't use it.", self.product_id());
                        Err(NearError::new(ErrorCode::NEAR_ERROR_NO_AVAILABLE, error_string))
                    }
                }
            })
    }
}

impl std::ops::Deref for ProductItem {
    type Target = Product_info;

    fn deref(&self) -> &Self::Target {
        &self.product
    }
}