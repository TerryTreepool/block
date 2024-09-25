
use std::{collections::HashMap, path::PathBuf};

use log::{trace, error, warn, info};

use near_base::{ErrorCode, NearError, thing::ThingObject, NearResult, FileEncoder, };
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use near_util::DESC_SUFFIX_NAME;
use protos::{hci::thing::{Thing_info, Thing_add}, DataContent, try_decode_raw_object, try_encode_raw_object};
use storage::ItemTrait;

use crate::{process::Process, caches::thing::ThingItemBuild};

pub struct AddThingRoutine {
    process: Process,
}

impl AddThingRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddThingRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("AddThingRoutine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Thing_add, req, o, { (o.take_thing(), o.take_thing_data()) }, { header_meta.sequence() });

        let r: DataContent<ThingObject> = match r {
            DataContent::Content((thing, thing_data)) => self.on_routine(header_meta, thing, thing_data).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, thing: Thing_info, thing_data: HashMap<String, String>) -> NearResult<ThingObject> {

        // check brand
        self.process.brand_storage().load_with_prefix(thing.brand_id()).await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;
        // check major & minor product
        {
            let major_pruduct = 
                self.process.product_storage().load_with_prefix(thing.major_product_id()).await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })?;

            let _ = 
                major_pruduct.children()
                    .products().iter()
                    .find(| item | item.product_id() == thing.minor_product_id())
                    .ok_or_else(|| {
                        let error_string = format!("Not found [{}]-[{}] product.", thing.major_product_id(), thing.minor_product_id());
                        error!("{error_string}, sequence: {}", header_meta.sequence());
                        NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                    })?;
        }

        // add thing
        // build thing object
        let thing = {
            ThingItemBuild {
                brand_id: thing.brand_id(),
                major_product_id: thing.major_product_id(),
                minor_product_id: thing.minor_product_id(),
                thing_name: thing.thing_name(),
                thing_mac: thing.mac_address(),
                thing_data,
            }
            .build()
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?
        };

        let ((thing, thing_object), newly) = 
            match   self.process
                        .thing_storage()
                        .create_new(&thing)
                        .await {
                Ok(_) => {
                    Ok((thing.split(), true))
                }
                Err(e) if e.errno() == ErrorCode::NEAR_ERROR_ALREADY_EXIST => {
                    let error_string = format!("[{}] has been exist.", thing.thing().mac_address());
                    warn!("{error_string}, sequence: {}", header_meta.sequence());

                    self.process
                        .thing_storage()
                        .load_with_prefix(thing.id()).await
                        .map(| thing | {
                            ((thing.split()), false)
                        })
                        .map_err(| e | {
                            error!("{e}, sequence: {}", header_meta.sequence());
                            e
                        })
                }
                Err(e) => {
                    let error_string = format!("failed add [{}] thing with err: {e}", thing.thing().mac_address());
                    error!("{error_string}, sequence: {}", header_meta.sequence());
                    Err(NearError::new(e.errno(), error_string))
                }
            }?;

        if newly {
            let thing_object_clone = thing_object.clone();
            let process = self.process.clone();
            let thing = thing;
            let header_meta = header_meta.clone();
            async_std::task::spawn(async move {
                let _ = 
                    // save
                    thing_object_clone.encode_to_file(
                        process.config()
                                .thing_data_path
                                .join(PathBuf::new()
                                                .with_file_name(thing.thing_id())
                                                .with_extension(DESC_SUFFIX_NAME))
                                .as_path(), 
                        false)
                        .map(| _ | {
                            info!("[{}] been wrote.", thing.thing_id());
                        })
                        .map_err(| e | {
                            let error_string = format!("failed encode-to-file thing-object with err: {e}");
                            error!("{error_string}, sequence: {}", header_meta.sequence());
                            e
                        });
            });
        }

        Ok(thing_object)
    }

}
