
use std::{collections::{HashMap, hash_map::Entry}, path::PathBuf, sync::Arc};

use flexi_logger::{DeferredNow, Record, LevelFilter, };

use log::{info, Log};
use near_base::{NearResult, NearError, ErrorCode};

use crate::{LogModuleConfig, LogLevel};

#[derive(Clone)]
pub struct LoggerImpl {
    config: LogModuleConfig,
    logger: Arc<Box<dyn Log>>,
}

impl LoggerImpl {
    pub fn build_logger(log_dir: PathBuf, config: LogModuleConfig) -> NearResult<Self> {
        let format_writer = |w: &mut dyn std::io::Write, now: &mut DeferredNow, record: &Record, | -> Result<(), std::io::Error> {
            write!(
                w,
                "[{}] {} [{:?}] [{}:{}] {}",
                now.now().format("%Y-%m-%d %H:%M:%S%.6f %:z"),
                record.level(),
                std::thread::current().id(),
                record.file().unwrap_or("<unnamed>"),
                record.line().unwrap_or(0),
                &record.args()
            )
        };

        let spec = 
            flexi_logger::LogSpecBuilder::from_module_filters(&[flexi_logger::ModuleFilter {
                    module_name: None,
                    level_filter: config.level.into(),
                }])
                .build();

        let mut logger = flexi_logger::Logger::with(spec);

        if config.file {

            logger = 
            logger.log_to_file(flexi_logger::FileSpec::default()
                                                .directory(log_dir)
                                                .discriminant(format!("{}_{}", config.name, std::process::id()))
                                                .suppress_timestamp())
                .rotate(flexi_logger::Criterion::Size(config.file_max_size), 
                        flexi_logger::Naming::Numbers,
                        match config.file_max_count {
                            0 => flexi_logger::Cleanup::Never,
                            _ => flexi_logger::Cleanup::KeepLogFiles(config.file_max_count as usize)
                        })
                .format_for_files(format_writer);
        }

        if config.console != LogLevel::Off {
            logger = logger.duplicate_to_stderr(config.console.into());
            logger = logger.format_for_stderr(format_writer);

            #[cfg(feature = "colors")]
            {
                logger = logger.format_for_stderr(cyfs_colored_default_format);
            }
        }

        let (logger, _) = logger.build().map_err(|e| {
            NearError::new(ErrorCode::NEAR_ERROR_3RD, format!("init logger failed! {}", e))
        })?;

        Ok(Self{
            config,
            logger: Arc::new(logger),
        })
    }

    pub fn clone_with_config(&self, config: LogModuleConfig) -> Self {
        Self {
            config,
            logger: self.logger.clone(),
        }
    }

    pub fn level(&self) -> LogLevel {
        self.config.level
    }
}

pub struct Logger {
    global: LoggerImpl,
    module: HashMap<String, LoggerImpl>,
}

impl Logger {

    pub fn new(log_dir: PathBuf, config: LogModuleConfig) -> NearResult<Self> {
        Ok(Self {
            global: LoggerImpl::build_logger(log_dir, config)?,
            module: Default::default(),
        })
    }

    pub fn add_module(&mut self, name: String, config: LogModuleConfig) {
        match self.module.entry(name) {
            Entry::Occupied(_exist) => {},
            Entry::Vacant(empty) => {
                empty.insert(self.global.clone_with_config(config));
            }
        }
    }

    // 屏蔽一些基础库的trace log等
    pub fn disable_async_std_log(&mut self) {
        let mod_list = [
            ("async_io", LogLevel::Warn),
            ("polling", LogLevel::Warn),
            ("async_tungstenite", LogLevel::Warn),
            ("tungstenite", LogLevel::Warn),
            ("async_std", LogLevel::Warn),
            ("tide", LogLevel::Warn),
            ("async-h1", LogLevel::Warn),
        ];

        mod_list.iter().for_each(|(name, level)| {
            let mut conf = LogModuleConfig::new_default(&name);
            conf.set_level(*level)
                .set_console(*level)
                .set_file(false);

            self.add_module(name.to_string(), conf);
        })
    }    

    pub fn start(self) -> NearResult<()> {
        let max_level = self.global.level();
        log::set_max_level(max_level.into());

        if let Err(e) = log::set_boxed_logger(self.into()) {
            Err(NearError::new(ErrorCode::NEAR_ERROR_3RD, format!("call set_boxed_logger failed! {}", e)))
        } else {
            println!("log max level: {}", max_level);
            Ok(())
        }
    }

    pub fn display_debug_info() {

        info!("current dir: {:?}", std::env::current_dir());

        // 输出环境信息，用以诊断一些环境问题
        for argument in std::env::args() {
            info!("arg: {}", argument);
        }

        for (key, value) in std::env::vars() {
            info!("env: {}: {}", key, value);
        }
    }

    pub fn get_logger_impl(&self, name: Option<&str>) -> &LoggerImpl {
        name.map(| name |{
            self.module.get(name).unwrap_or(&self.global)
        })
        .unwrap_or(&self.global)
    }

    pub fn flush() {
        log::logger().flush();
    }
}

impl Into<Box<dyn Log>> for Logger {
    fn into(self) -> Box<dyn Log> {
        Box::new(self) as Box<dyn Log>
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let target = {
            let target = metadata.target();
            match target.find("::") {
                Some(pos) => Some(&target[..pos]),
                None => None
            }
        };

        let meta_level = metadata.level() as usize;
        let level = self.get_logger_impl(target).level() as usize;

        level >= meta_level
    }

    fn flush(&self) {
        self.global.logger.flush();
    }

    fn log(&self, record: &Record) {
        let target = record.metadata().target();

        let target = match target.find("::") {
            Some(pos) => { Some(&target[..pos]) }
            None => Some(target)
        };

        let logger = self.get_logger_impl(target);
        let level: LevelFilter = logger.level().into();
        if record.metadata().level().le(&level) {
            logger.logger.log(record);
        }
    }
}
