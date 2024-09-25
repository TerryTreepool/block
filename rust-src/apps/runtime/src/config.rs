use std::{path::PathBuf,
          sync::RwLock, time::Duration,
    };
use near_core::{get_data_path, LogLevel};

struct RuntimeConfigImpl {
    pub extention_name: Option<String>,
    pub core_desc: PathBuf,
    pub device_desc: PathBuf,
    pub log_level: LogLevel,
    pub wait_online_timeout: Duration,
}

pub struct RuntimeConfig(RwLock<RuntimeConfigImpl>);

impl std::default::Default for RuntimeConfig {
    fn default() -> Self {
        Self(RwLock::new(RuntimeConfigImpl {
            extention_name: None,
            core_desc: PathBuf::default(),
            device_desc: PathBuf::default(),
            log_level: LogLevel::default(),
            wait_online_timeout: Duration::from_secs(1000),
        }))
    }
}

impl RuntimeConfig {
    pub fn extention_name(&self) -> String {
        self.0.read().unwrap().extention_name.clone().unwrap_or("unnamed".to_owned())
    }

    pub fn set_core_desc(&self, core_name: &str) {
        self.0.write().unwrap().core_desc = get_data_path().join(format!("{}.desc", core_name));
    }

    pub fn core_desc(&self) -> PathBuf {
        self.0.read().unwrap().core_desc.clone()
    }

    pub fn set_device_desc(&self, device_name: &str) {
        let mut_self = &mut *self.0.write().unwrap();
        mut_self.extention_name = Some(device_name.to_owned());
        mut_self.device_desc = get_data_path().join(format!("{}.desc", device_name));
    }
    pub fn device_desc(&self) -> PathBuf {
        self.0.read().unwrap().device_desc.clone()
    }

    pub fn set_log_level(&self, log_level: LogLevel) {
        self.0.write().unwrap().log_level = log_level;
    }
    pub fn log_level(&self) -> LogLevel {
        self.0.read().unwrap().log_level.clone()
    }

    pub fn set_wait_online_timeout(&self, timeout: Duration) {
        self.0.write().unwrap().wait_online_timeout = timeout;
    }
    pub fn wait_online_timeout(&self) -> Duration {
        self.0.read().unwrap().wait_online_timeout
    }
}

lazy_static::lazy_static! {
    pub static ref RUNTIME_CONFIG: RuntimeConfig = RuntimeConfig::default();
}
