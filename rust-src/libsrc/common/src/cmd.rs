
use std::{path::PathBuf, str::FromStr};

use clap::{Arg, ArgMatches, };
use near_core::{get_data_path, LogLevel};
use near_util::{DESC_SUFFIX_NAME, KEY_SUFFIX_NAME};

use crate::{RunMode, config::{action, service_name}, action::{ProcessActionInner, DAEMON_FLAG, CONSOLE_FLAG}, };

pub struct ArgStruct<'arg> {
    pub(crate) name: &'arg str,
    pub(crate) long: Option<&'arg str>,
    pub(crate) short: Option<char>,
    pub(crate) help: Option<&'arg str>,
    pub(crate) must_value: bool,
}

impl<'arg> ArgStruct<'arg> {
    pub(crate) fn into_arg(&self) -> Arg<'arg> {
        let mut arg = Arg::new(self.name);

        if let Some(long) = self.long {
            arg = arg.long(long);
        }

        if let Some(short) = self.short {
            arg = arg.short(short);
        }

        if let Some(help) = self.help {
            arg = arg.help(help)
        }

        arg.takes_value(self.must_value)
    }
}

lazy_static::lazy_static!{
    pub(crate) static ref CORE_ARG: ArgStruct<'static> = ArgStruct {
        name: "core",
        long: Some("core"),
        short: Some('C'),
        help: Some("core-service desc info"),
        must_value: true,
    };

    pub(crate) static ref DESC_ARG: ArgStruct<'static> = ArgStruct {
        name: "desc",
        long: Some("desc"),
        short: None,
        help: Some("runtime-service desc info"),
        must_value: true,
    };
    
    pub(crate) static ref DAEMON_ARG: ArgStruct<'static> = ArgStruct {
        name: "daemon",
        long: Some("daemon"),
        short: Some('D'),
        help: Some("run as daemon"),
        must_value: false,

    };

    pub(crate) static ref QUIT_ARG: ArgStruct<'static> = ArgStruct {
        name: "quit",
        long: Some("quit"),
        short: Some('Q'),
        help: Some("quit process"),
        must_value: false,

    };

    pub(crate) static ref LOGLEVEL_ARG: ArgStruct<'static> = ArgStruct {
        name: "log-level",
        long: Some("log-level"),
        short: None,
        help: Some("set [trace, debug, info, warn, error] level"),
        must_value: true,
    };

    pub(crate) static ref CORE_SERVICE_CFG: String = "core-service".to_owned();
}

pub fn build_core_arg() -> Vec<Arg<'static>> {
    vec![CORE_ARG.into_arg()]
}

pub fn build_runtime_arg() -> Vec<Arg<'static>> {
    vec![CORE_ARG.into_arg(), DESC_ARG.into_arg()]
}

// pub fn build_signal_command() -> Command<'static> {
//     Command::new("signal")
//         .about("signal process daemon or quit.")
//         .arg(DAEMON_ARG.into_arg())
//         .arg(QUIT_ARG.into_arg())
// }

// pub fn check_runtime_command_with_args(matches: &ArgMatches) {

//     if matches.get_flag(CORE_ARG.name) {
//         set_daemon(ProcessActionInner::Daemon);
//     }

// }

pub(crate) fn check_action_with_args(matches: &ArgMatches) -> ProcessActionInner {
    if matches.contains_id(DAEMON_ARG.name) {
        ProcessActionInner::Start(DAEMON_FLAG as u16)
    } else if matches.contains_id(QUIT_ARG.name) {
        ProcessActionInner::Stop
    } else {
        ProcessActionInner::Start(CONSOLE_FLAG as u16)
    }
    // let action = if let Some(matches) = matches.subcommand_matches("signal") {
    //     if matches.contains_id(DAEMON_ARG.name) {
    //         ProcessActionInner::Start(DAEMON_FLAG as u16)
    //     } else if matches.contains_id(QUIT_ARG.name) {
    //         ProcessActionInner::Stop
    //     } else {
    //         ProcessActionInner::Start(CONSOLE_FLAG as u16)
    //     }
    // } else {
    //     ProcessActionInner::Start(DAEMON_FLAG as u16)
    // };

    // action
}

pub(crate) fn check_core_command_with_args(matches: &ArgMatches) -> RunMode {

    let mode = match action() {
        ProcessActionInner::Start(_) => {
            let core_name = if let Some(core) = matches.value_of(CORE_ARG.name) {
                core
            } else {
                CORE_SERVICE_CFG.as_str()
            };
            RunMode::Core(PathBuf::from(get_data_path().join(format!("{}.{DESC_SUFFIX_NAME}", core_name))),
                          PathBuf::from(get_data_path().join(format!("{}.{KEY_SUFFIX_NAME}", core_name))))
        }
        ProcessActionInner::Stop => {
            RunMode::Unknown
        }
    };

    mode
}

pub(crate) fn check_log_level_command_with_args(matches: &ArgMatches) -> Option<LogLevel> {
    let level = if let Some(level) = matches.value_of(LOGLEVEL_ARG.name) {
            if let Ok(level) = LogLevel::from_str(level) {
                Some(level)
            } else {
                None
            }
    } else {
        None
    };

    level
}

pub(crate) fn check_runtime_command_with_args(matches: &ArgMatches) -> RunMode {

    let default_name = service_name();

    let mode = match action() {
        ProcessActionInner::Start(_) => {
            let core_name = if let Some(core) = matches.value_of(CORE_ARG.name) {
                core
            } else {
                CORE_SERVICE_CFG.as_str()
            };

            let runtime_name = if let Some(desc) = matches.value_of(DESC_ARG.name) {
                desc
            } else {
                default_name.as_str()
            };

            RunMode::Runtime(PathBuf::from(get_data_path().join(format!("{}.{DESC_SUFFIX_NAME}", core_name))), 
                             PathBuf::from(get_data_path().join(format!("{}.{DESC_SUFFIX_NAME}", runtime_name))))
            
        }
        ProcessActionInner::Stop => {
            RunMode::Unknown
        }
    };

    mode

}