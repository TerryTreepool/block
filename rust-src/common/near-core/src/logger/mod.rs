
pub mod log_builder;
pub mod log_config;
pub mod log_logger;

use std::str::FromStr;

pub use log_builder::LoggerBuilder;
pub use log_config::LogModuleConfig;

use near_base::{NearError, ErrorCode};

#[repr(usize)]
#[derive(Copy, Eq, PartialEq, PartialOrd, Ord, Clone, Debug, Hash, )]
pub enum LogLevel {
    Off = 0,
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        {Self::Debug}

        #[cfg(not(debug_assertions))]
        {Self::Info}
    }
}

impl FromStr for LogLevel {
    type Err = NearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("trace") {
            Ok(LogLevel::Trace)
        } else if s.eq_ignore_ascii_case("debug") {
            Ok(LogLevel::Debug)
        } else if s.eq_ignore_ascii_case("info") {
            Ok(LogLevel::Info)
        } else if s.eq_ignore_ascii_case("warn") {
            Ok(LogLevel::Warn)
        } else if s.eq_ignore_ascii_case("error") {
            Ok(LogLevel::Error)
        } else if s.eq_ignore_ascii_case("off") {
            Ok(LogLevel::Off)
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("{} is unknow", s)))
        }
    }
}

impl std::fmt::Display for LogLevel {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match *self {
            Self::Off   => "off",
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info  => "info",
            Self::Warn  => "warn",
            Self::Error => "error",
        };
        write!(f, "{}", level)
    }

}

impl Into<flexi_logger::Duplicate> for LogLevel {

    fn into(self) -> flexi_logger::Duplicate {
        match self {
            Self::Off   => flexi_logger::Duplicate::None,
            Self::Trace => flexi_logger::Duplicate::Trace,
            Self::Debug => flexi_logger::Duplicate::Debug,
            Self::Info  => flexi_logger::Duplicate::Info,
            Self::Warn  => flexi_logger::Duplicate::Warn,
            Self::Error => flexi_logger::Duplicate::Error,
        }        
    }
}

impl Into<flexi_logger::LevelFilter> for LogLevel {

    fn into(self) -> flexi_logger::LevelFilter {
        match self {
            Self::Off   => flexi_logger::LevelFilter::Off,
            Self::Trace => flexi_logger::LevelFilter::Trace,
            Self::Debug => flexi_logger::LevelFilter::Debug,
            Self::Info  => flexi_logger::LevelFilter::Info,
            Self::Warn  => flexi_logger::LevelFilter::Warn,
            Self::Error => flexi_logger::LevelFilter::Error,
        }        
    }
}

#[macro_export]
macro_rules! near_error {
    ( $err_code: expr, $err_message: expr ) => {
        {
            let err = NearError::new($err_code, $err_message);
            log::error!("{err}");
            err
        }
    };
}
