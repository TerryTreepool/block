use once_cell::sync::OnceCell;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused)]
#[allow(improper_ctypes)]
mod bindgen;

pub mod management;

pub struct Config {
    timeout_interval: u16,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            timeout_interval: 10000,
        }
    }
}

impl Config {
    pub fn get_instace() -> &'static Config {
        static INSTANCE: OnceCell<Config> = OnceCell::<Config>::new();

        INSTANCE.get_or_init(|| {
            let c = Config::default();
            c
        })
    }
}
