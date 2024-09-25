
use near_base::{ErrorCode, NearError, NearResult};

use protos::{brand::Brand_info, 
             product::Product_info, 
             device::Device_info, 
             thing_group::Thing_group_info};
use topic_util::types::brand_types::Status;

use super::CheckTrait;

impl CheckTrait for Brand_info {
    fn check_status(&self) -> NearResult<()> {
        self.status
            .try_into()
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

impl CheckTrait for Product_info {
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

impl CheckTrait for Device_info {
    fn check_status(&self) -> NearResult<()> {
        self.status.try_into()
            .map_or_else(| _ |{
                let error_string = format!("{}'s status is exception, can't use it.", self.device_id());
                Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, error_string))
            },| status | {
                match status {
                    Status::Eanbled => Ok(()),
                    Status::Disabled => {
                        let error_string = format!("{} brand is diabled, cann't use it.", self.device_id());
                        Err(NearError::new(ErrorCode::NEAR_ERROR_NO_AVAILABLE, error_string))
                    }
                }
            })
    }
}

impl CheckTrait for Thing_group_info {
    fn check_status(&self) -> NearResult<()> {
        self.status.try_into()
            .map_or_else(| _ |{
                let error_string = format!("{}'s status is exception, can't use it.", self.group_id());
                Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, error_string))
            },| status | {
                match status {
                    Status::Eanbled => Ok(()),
                    Status::Disabled => {
                        let error_string = format!("{} status is diabled, cann't use it.", self.group_id());
                        Err(NearError::new(ErrorCode::NEAR_ERROR_NO_AVAILABLE, error_string))
                    }
                }
            })
    }
}
