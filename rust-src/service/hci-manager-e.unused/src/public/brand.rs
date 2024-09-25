
use near_base::{NearResult, NearError, ErrorCode};

use dataagent_util::Helper;
use protos::brand::Brand_info;

pub async fn get_brand(db: &Helper, brand_id: &str) -> NearResult<Brand_info> {
    let mut brand = Default::default();

    db.query_all_with_param::<Brand_info>(crate::p::GET_BRAND.0, 
                                        Brand_info {
                                            brand_id: brand_id.to_owned(),
                                            ..Default::default()
                                        })
        .await?
        .get_mut(0)
        .map(| item | {
            std::mem::swap(item, &mut brand);
            brand
        })
        .ok_or_else(|| {
            NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found [{}] brand.", brand_id))
        })
}
