
use std::{sync::RwLock, path::PathBuf};

use near_core::LogLevel;

use crate::action::ProcessActionInner;

lazy_static::lazy_static!{
    static ref CONFIG: RwLock<NearConfig> = RwLock::new(NearConfig::default());
}

#[derive(Clone)]
pub struct NearConfig {
    pub(crate) service_name: String,
    pub(crate) mode: RunMode,
    pub(crate) log_level: LogLevel,
    pub(crate) action: ProcessActionInner,
}

impl std::default::Default for NearConfig {
    fn default() -> Self {
        Self {
            service_name: Default::default(),
            mode: RunMode::Unknown,
            #[cfg(debug_assertions)]
            log_level: LogLevel::Trace,
            #[cfg(not(debug_assertions))]
            log_level: LogLevel::Info,
            action: ProcessActionInner::Start(0),
        }
    }
}

impl NearConfig {
    #[inline]
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

}

#[derive(Clone)]
pub enum RunMode {
    Unknown,
    Core(PathBuf  /* core */, PathBuf /* private key */),
    Runtime(PathBuf /* core */, PathBuf /* desc */),
    Aux,
}

#[allow(unused)]
pub(crate) fn set_service_name(name: &str) {
    CONFIG.write().unwrap().service_name = name.to_owned();
}

#[allow(unused)]
pub(crate) fn service_name() -> String {
    CONFIG.read().unwrap().service_name().to_owned()
}

#[allow(unused)]
pub(crate) fn set_log_level(level: LogLevel) {
    CONFIG.write().unwrap().log_level = level;
}

#[allow(unused)]
pub(crate) fn log_level() -> LogLevel {
    CONFIG.read().unwrap().log_level
}

#[allow(unused)]
pub(crate) fn set_mode(mode: RunMode) {
    CONFIG.write().unwrap().mode = mode;
}

#[allow(unused)]
pub(crate) fn mode() -> RunMode {
    CONFIG.read().unwrap().mode.clone()
}

#[allow(unused)]
pub(crate) fn set_action(action: ProcessActionInner) {
    CONFIG.write().unwrap().action = action;
}

#[allow(unused)]
pub(crate) fn action() -> ProcessActionInner {
    CONFIG.read().unwrap().action
}

#[allow(unused)]
pub(crate) fn config() -> NearConfig {
    CONFIG.read().unwrap().clone()
}
