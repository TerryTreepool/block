
use near_base::{NearResult, NearError, ErrorCode};

use dataagent_util::Helper;
use protos::product::Product_info;

pub async fn get_product(db: &Helper, product_id: &str) -> NearResult<Product_info> {
    let mut product = Default::default();

    db.query_all_with_param::<Product_info>(crate::p::GET_PRODUCT.0, 
                                        Product_info {
                                            product_id: product_id.to_owned(),
                                            ..Default::default()
                                        })
        .await?
        .get_mut(0)
        .map(| item | {
            std::mem::swap(item, &mut product);
            product
        })
        .ok_or_else(|| {
            NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found [{}] product.", product_id))
        })
}
