
use std::sync::Arc;

use near_base::Sequence;
use near_core::get_data_path;
use rlua::{UserData, UserDataMethods, AnyUserData};

struct ConfigureDataImpl {
    serial_num: Sequence,
}

#[derive(Clone)]
pub struct ConfigureData(Arc<ConfigureDataImpl>);

impl ConfigureData {

    pub fn get_instace() -> &'static ConfigureData {

        static INSTANCE: once_cell::sync::OnceCell<ConfigureData> = once_cell::sync::OnceCell::new();

        INSTANCE.get_or_init(|| {
            let c = ConfigureData::new();
            c
        })

    }

    fn new() -> ConfigureData {
        ConfigureData(Arc::new(ConfigureDataImpl{
            serial_num: Sequence::random(),
        }))
    }

    pub fn gen_serial_num(&self) -> u32 {
        self.0.serial_num.generate().into_value()
    }

    pub fn mem_cpy(&self, src: &Vec<u8>, des: &mut Vec<u8>) {
        des.copy_from_slice(src.as_slice());
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
        // methods.add_function("set_value", |_, (ud, value): (AnyUserData, u8)| {
        //     ud.borrow_mut::<MyUserdata>()?.id = value;
        //     Ok(())
        // });
        // methods.add_function("get_constant", |_, ()| Ok(7));
    }
}

