
use async_std::path::PathBuf;
use near_core::get_data_path;
use near_util::DESC_SUFFIX_NAME;
use once_cell::sync::OnceCell;

use near_base::{DeviceObject, FileDecoder};

pub struct Configures {
    pub(crate) desc: DeviceObject,
}

impl Configures {
    pub fn get_instance() -> &'static Self {
        static INSTACNE: OnceCell<Configures> = OnceCell::new();
        INSTACNE.get_or_init(||{
            let desc = 
            {
                let core_path = get_data_path().join(PathBuf::new().with_file_name("core-service").with_extension(DESC_SUFFIX_NAME));
                println!("from-desc: {}", core_path.display());
                DeviceObject::decode_from_file(core_path.as_path()).expect(&format!("failed get [{}]", core_path.display()))
            };

            Self {
                desc
            }
        })
    }
}
