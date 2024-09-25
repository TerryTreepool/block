
use std::{str::FromStr, path::PathBuf};

use log::{trace, error, info};

use near_base::{ErrorCode, NearError, FileEncoder, thing::ThingObject, NearResult, };
use near_core::time_utils::native_now;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use near_util::ThingBuilder;
use protos::{device::Device_info, DataContent, try_decode_raw_object, try_encode_raw_object, hci::Hci_add_thing};
use topic_util::types::brand_types::Status;
use common::RuntimeStack;
use dataagent_util::Transaction;

use crate::{process::Process, public::CheckTrait};

pub struct AddDeviceRoutine {
    process: Process,
}

impl AddDeviceRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddDeviceRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddDeviceRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("add device routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Hci_add_thing, req, o, o, { header_meta.sequence() });

        let r: DataContent<ThingObject> = match r {
            DataContent::Content(thing) => self.on_routine(header_meta, thing).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddDeviceRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, mut thing: Hci_add_thing) -> NearResult<ThingObject> {

        let mut trans = 
            self.process()
                .db_helper()
                .begin_transaction()
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        let thing_name = thing.thing_name().trim();
        if thing_name.is_empty() {
            let error_string = "thing name is empty";
            error!("{error_string}, sequence = {}", header_meta.sequence());
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, error_string))
        } else {
            Ok(())
        }?;

        // check brand
        crate::public::brand::get_brand(self.process().db_helper(), 
                                        thing.brand_id())
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?
            .check_status()?;


        // check product
        crate::public::product::get_product(self.process().db_helper(),
                                            thing.product_id())
            .await
            .map_err(| e |{
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?
            .check_status()?;

        // add thing
        // build thing object
        let (thing_id, thing_object) = {
            let mac = {
                let mac = 
                    mac_address::MacAddress::from_str(thing.thing().mac_address())
                        .map_err(| e | {
                            let error_string = format!("failed parse to mac-address with err: {e}");
                            error!("{error_string}, sequence: {}", header_meta.sequence());
                            NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, error_string)
                        })?;
                mac
            };

            let thing = 
                ThingBuilder::new()
                    .owner(Some(RuntimeStack::get_instance().stack().core_device().object_id()))
                    .mac_address(mac.bytes())
                    .owner_depend_id(thing.brand_id().to_owned())
                    .user_data(thing.mut_thing().take_data())
                    .build()
                    .map_err(| e | {
                        let error_string = format!("failed build thing-object with err: {e}");
                        error!("{error_string}, sequence: {}", header_meta.sequence());
                        e
                    })?;

            (thing.object_id().to_string(), thing)
        };

        {
            // write to db
            let now = native_now().format("%Y-%m-%d %H:%M:%S").to_string();
            let device = Device_info {
                brand_id: thing.take_brand_id(),
                product_id: thing.take_product_id(),
                device_id: thing_id.clone(),
                device_name: thing.take_thing_name(),
                mac_address: thing.mut_thing().take_mac_address(),
                begin_time: now.clone(),
                update_time: now,
                status: Status::Eanbled.into(),
                ..Default::default()
            };

            self.update_thing(&mut trans, header_meta, device)
                .await?;
        }

        // save
        thing_object.encode_to_file(
            self.process()
                      .config()
                      .thing_data_path
                      .join(PathBuf::new()
                                    .with_file_name(thing_id.as_str())
                                    .with_extension("desc"))
                      .as_path(), 
            false)
            .map(| _ | {
                info!("[{}] been wrote.", thing_id.as_str());
            })
            .map_err(| e | {
                let error_string = format!("failed encode-to-file thing-object with err: {e}");
                error!("{error_string}, sequence: {}", header_meta.sequence());
                e
            })?;

        trans.commit()
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        Ok(thing_object)
    }

    async fn update_thing(&self, trans: &mut Transaction, header_meta: &HeaderMeta, mut device: Device_info) -> NearResult<()> {
        enum NextStep {
            New(Device_info),
            Update(Device_info),
        }

        let next_step = 
        match crate::public::thing::get_thing(self.process().db_helper(), device.device_id())
                .await {
            Ok(mut thing) => {
                thing.set_status(device.status);
                thing.set_update_time(device.take_update_time());
                Ok(NextStep::Update(thing))
            }
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_NOTFOUND => Ok(NextStep::New(device)),
                    _ => {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        Err(e)
                    }
                }
            }
        }?;

        match next_step {
            NextStep::New(thing) => 
                trans.execute_with_param(crate::p::ADD_DEVICE.0, &thing)
                    .await
                    .map_err(| e |{
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    }),
            NextStep::Update(thing) => 
                trans.execute_with_param(crate::p::UPDATE_DEVICE.0, &thing)
                    .await
                    .map_err(| e |{
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    }),
        }
    }

}
