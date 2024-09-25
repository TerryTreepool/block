
use std::{sync::Arc, collections::HashMap};

use common::RuntimeStack;
use log::{debug, info};
use mac_address::MacAddress;
use near_base::Sequence;
use near_core::get_data_path;
use rlua::{UserData, UserDataMethods, AnyUserData};

use crate::{process::Process, cache::ThingStatus};

struct ConfigureDataImpl {
    serial_num: Sequence,
}

static INSTANCE: once_cell::sync::OnceCell<ConfigureData> = once_cell::sync::OnceCell::new();

#[derive(Clone)]
pub struct ConfigureData(Arc<ConfigureDataImpl>);

impl ConfigureData {

    pub fn get_instace() -> &'static ConfigureData {
        INSTANCE.get().unwrap()
    }

    pub fn init() {
        let data = ConfigureData(Arc::new(ConfigureDataImpl{
            serial_num: Sequence::random(),
        }));

        let _ = INSTANCE.set(data);
    }
}

impl ConfigureData {

    pub fn gen_serial_num(&self) -> u32 {
        self.0.serial_num.generate().into_value()
    }

    pub fn mem_cpy(&self, src: &Vec<u8>, des: &mut Vec<u8>) {
        des.copy_from_slice(src.as_slice());
    }

    pub fn core_id(&self) -> String {
        let core_id = 
        hex::encode(
            RuntimeStack::get_instance()
                .stack()
                .core_device()
                .object_id()
                .as_ref()
                .as_slice()[0..6]
                .to_vec()
        );

        debug!("+++++++++++++++++++++++++++++++core-id: {core_id}");

        core_id

    }

    pub fn get_thingdata(&self, mac: String) -> HashMap<String, String> {
        let data = 
            if let Ok(mac) = mac.parse::<MacAddress>() {
                if let Ok(thing) = Process::get_instance().thing_components().get_thing_by_mac(mac.bytes()) {
                    match thing.status() {
                        ThingStatus::Disable => { info!("mac: {} has been disabled, ignore.", mac.to_string()); None },
                        _ => Some(thing.thing().body().content().user_data().clone()),
                    }
                } else {
                    None
                }
            } else {
                None
            };

        data.unwrap_or_default()
    }
}

impl UserData for ConfigureData {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("gen_serial_num", |_, ud: AnyUserData| {
            Ok(ud.borrow::<ConfigureData>()?.gen_serial_num())
        });

        methods.add_function("mem_cpy", |_, (ud, src, mut des): (AnyUserData, Vec<u8>, Vec<u8>) | {
            let r = ud.borrow::<ConfigureData>()?.mem_cpy(&src, &mut des);
            Ok(r)
        });

        methods.add_function("get_project_path", |_, _: AnyUserData | {
            let r = get_data_path().to_string_lossy().to_string();
            Ok(r)
            // Ok(r)
        });

        methods.add_function("core_mac", |_, ud: AnyUserData | {
            Ok(ud.borrow::<ConfigureData>()?.core_id())
        });

        methods.add_function("get_thingdata", |_, (ud, mac): (AnyUserData, String) | {
            Ok(ud.borrow::<ConfigureData>()?.get_thingdata(mac))
        });

    }
}

