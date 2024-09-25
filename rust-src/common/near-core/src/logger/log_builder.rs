
use std::{path::PathBuf, collections::{HashMap, hash_map::Entry}, str::FromStr};

use near_base::NearResult;

use crate::{log_config::LogModuleConfig, LogLevel, log_logger::Logger, };

pub struct LoggerBuilder {
    log_dir: PathBuf,
    global: LogModuleConfig,
    module: HashMap<String, LogModuleConfig>,
}

impl LoggerBuilder {
    pub fn new(name: &str, log_dir: PathBuf) -> Self {

        Self {
            log_dir,
            global: LogModuleConfig::new_default(name),
            module: HashMap::new(),
        }
    }

    pub fn set_level(mut self, level: LogLevel) -> Self {
        self.global.set_level(level);
        self
    }

    pub fn set_console(mut self, level: LogLevel) -> Self {
        self.global.set_console(level);
        self
    }

    pub fn set_file(mut self, file: bool) -> Self {
        self.global.set_file(file);
        self
    }

    pub fn set_file_max_size(mut self, file_max_size: u64) -> Self {
        self.global.set_file_max_size(file_max_size);
        self
    }

    pub fn set_file_max_count(mut self, file_max_count: u32) -> Self {
        self.global.set_file_max_count(file_max_count);
        self
    }

    pub fn add_module(mut self, module_name: &str, level: Option<&str>, console_level: Option<&str>) -> Self {

        let mut module_config = LogModuleConfig::new_default(module_name);

        module_config
            .set_level( level.and_then(| v | LogLevel::from_str(v).ok()).unwrap_or(LogLevel::Off) )
            .set_console( console_level.and_then(| v | LogLevel::from_str(v).ok()).unwrap_or(LogLevel::Off) );

        match self.module.entry(module_name.to_owned()) {
            Entry::Occupied(_exist) => {
                
            }
            Entry::Vacant(empty) => {
                empty.insert(module_config);
            }
        }
        
        self
    }
}

impl LoggerBuilder {
    pub fn build(self) -> NearResult<()> {
        let mut logger = Logger::new(self.log_dir, self.global)?;

        logger.disable_async_std_log();

        let mut module = HashMap::new();

        for (name, config) in std::mem::replace(&mut module, self.module) {
            logger.add_module(name, config)
        }

        logger.start()
    }
}
