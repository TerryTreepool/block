
use near_base::{NearResult, NearError, ErrorCode};

use protos::brand::Brand_info;
use topic_util::types::brand_types::Status;

use super::manager_template::{ItemTrait, UpdateItemTrait, CheckTrait, UpdateItemTrait_V2};

#[derive(Clone)]
pub struct BrandItem {
    brand_info: Brand_info
}

impl From<BrandItem> for Brand_info {
    fn from(value: BrandItem) -> Self {
        value.brand_info
    }
}

impl From<Brand_info> for BrandItem {
    fn from(value: Brand_info) -> Self {
        Self{
            brand_info: value,
        }
    }
}

impl ItemTrait for BrandItem {
    fn get_item_id(&self) -> &str {
        self.brand_info.brand_id()
    }
}

impl UpdateItemTrait<BrandItem> for BrandItem {
    fn update_item(&mut self, new_item: BrandItem) {
        let mut_brand_info = &mut self.brand_info;

        debug_assert_eq!(mut_brand_info.brand_id(), new_item.brand_info.brand_id());

        mut_brand_info.set_update_time(new_item.brand_info.update_time);
        mut_brand_info.set_status(new_item.brand_info.status);

    }
}

// impl UpdateItemTrait_V2<Brand_info, Brand_info> for BrandItem {
//     fn update_item<CB: Fn(&Brand_info)>(&mut self, context: Brand_info, cb: CB) -> NearResult<()> {

//         debug_assert_eq!(self.brand_id(), context.brand_id());

//         if self.brand_id() != context.brand_id() {
//             Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "fatal group id"))
//         } else {
//             Ok(())
//         }?;

//         if self.status == context.status {
//             Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, "Ingore status"))
//         } else {
//             Ok(())
//         }?;

//         self.set_status(context.status);
//         self.set_update_time(context.update_time);

//         cb(&self.brand_info);

//         Ok(())
//     }
// }

impl CheckTrait for BrandItem {
    fn check_status(&self) -> NearResult<()> {
        self.brand_info.status.try_into()
            .map_or_else(| _ |{
                let error_string = format!("{}'s status is exception, can't use it.", self.brand_id());
                Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, error_string))
            },| status | {
                match status {
                    Status::Eanbled => Ok(()),
                    Status::Disabled => {
                        let error_string = format!("{} brand is diabled, cann't use it.", self.brand_id());
                        Err(NearError::new(ErrorCode::NEAR_ERROR_NO_AVAILABLE, error_string))
                    }
                }
            })
    }
}

impl std::ops::Deref for BrandItem {
    type Target = Brand_info;

    fn deref(&self) -> &Self::Target {
        &self.brand_info
    }
}

impl std::ops::DerefMut for BrandItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.brand_info
    }
}
