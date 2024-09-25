
use log::{trace, warn, error};

use near_base::{NearResult, ErrorCode, NearError};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{hci::brand::{Brand_add, Brand_info}, try_decode_raw_object, DataContent, try_encode_raw_object};

use crate::{process::Process, caches::brand::BrandItem};

pub struct AddBrandRoutine {
    process: Process,
}

impl AddBrandRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddBrandRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {

        trace!("add brand routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Brand_add, req, body, { body.take_brand() }, { header_meta.sequence() });

        let r: DataContent<Brand_info> = match r {
            DataContent::Content(brand) => self.on_routine(header_meta, brand).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, mut brand: Brand_info) -> NearResult<Brand_info> {

        let brand = 
            BrandItem::create_new(brand.take_brand_name())
                .map_err(| e | {
                    error!("faild create brand with err: {e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        self.process
            .brand_storage()
            .create_new(&brand)
            .await
            .map_err(| e | {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_ALREADY_EXIST => {
                        let error_string = format!("[{}] has been exist.", brand.brand_name());
                        warn!("{error_string}, sequence: {}", header_meta.sequence());
                        NearError::new(e.errno(), error_string)
                    }
                    _ => {
                        let error_string = format!("failed add {} brand with err: {e}", brand.brand_name());
                        warn!("{error_string}, sequence: {}", header_meta.sequence());
                        NearError::new(e.errno(), error_string)
                    }
                }
            })?;

        Ok(brand.into())
    }
}
