
use std::collections::BTreeMap;

use clap::{App, Arg};

use near_base::NearError;

const STOP_COMMAND: (&str, CommandValue) = ("stop", CommandValue{short_name: None, long_name: "stop", exist: false} );
const UPDATE_COMMAND: (&str, CommandValue) = ("update", CommandValue { short_name: None, long_name: "update", exist: false } );

macro_rules! DAEMON_COMMAND {
    ($name: expr, ($long_name: expr, $exist: expr)) => {
        Arg::with_name($name)
            .long($long_name)
            .takes_value($exist)
    };
    ($name: expr, ($short_name: expr, $long_name: expr, $exist: expr)) => {
        Arg::with_name($name)
            .short($short_name)
            .long($long_name)
            .takes_value($exist)
    };
}

pub enum Value<'a> {
    Present(bool),
    Value(&'a str),
}

struct CommandValue<'a> {
    short_name: Option<char>,
    long_name: &'a str,
    exist: bool,
}

pub struct ProcessCommandBuild<'a> {
    service_name: &'a str,
    author_name: &'a str,
    commands: BTreeMap<&'a str, CommandValue<'a>>,
}

impl<'a> ProcessCommandBuild<'a> {
    pub fn new(service_name: &'a str) -> Self {
        Self {
            service_name,
            author_name: "None",
            commands: BTreeMap::new(),
        }
    }

    pub fn author(mut self, author: &'a str) -> Self {
        self.author_name = author;
        self
    }

    pub fn append(mut self, name: &'a str, context: (Option<char>, &'a str, bool)) -> Self {
        let (short_name, long_name, exist) = context;

        let _ = self.commands
                    .entry(name)
                    .or_insert(CommandValue{
                        short_name,
                        long_name,
                        exist,
                    });
        self
    }

    pub fn launch<F: Fn(&str, Value)>(self, op: F) -> ProcessAction {
        let mut args = vec![];

        let app = App::new(self.service_name)
                            .author(self.author_name)
                            .arg(
                                Arg::with_name(STOP_COMMAND.0)
                                    .long(STOP_COMMAND.1.long_name)
                                    .takes_value(STOP_COMMAND.1.exist)
                            )
                            .arg(
                                Arg::with_name(UPDATE_COMMAND.0)
                                    .long(UPDATE_COMMAND.1.long_name)
                                    .takes_value(UPDATE_COMMAND.1.exist)
                            )
                            .args(
                                {
                                    self.commands.iter().for_each(|(&name, value)| {
                                        if let Some(short_name) = value.short_name {
                                            args.push(DAEMON_COMMAND!(name, (short_name, value.long_name, value.exist)));
                                        } else {
                                            args.push(DAEMON_COMMAND!(name, (value.long_name, value.exist)));
                                        }
                                    });
                                    args.as_slice()
                                }
                            );

        let matches = app.get_matches();

        if matches.is_present(STOP_COMMAND.0) {
            ProcessAction::Stop
        } else if matches.is_present(UPDATE_COMMAND.0) {
            ProcessAction::Update
        } else {
            self.commands.iter().for_each(| (&name, value) | {
                if value.exist {
                    if let Some(v) = matches.value_of(name) {
                        op(name, Value::Value(v))
                    }
                } else {
                    op(name, Value::Present(matches.is_present(name)))
                }
            });

            daemon::launch_as_daemon(self.service_name)
        }
    }
}

pub enum ProcessAction {
    Start,
    Stop,
    Update,
    Exit(NearError),
}

// #[cfg(unix)]
// pub mod daemon {
//     use cyfs_base::BuckyError;

//     use std::process::{exit, Command};

//     use nix::{
//         sys::wait::{waitpid, WaitStatus},
//         unistd::{fork, setsid, ForkResult},
//     };

//     pub fn launch_as_daemon(cmd_line: &str) -> Result<(), BuckyError> {
//         let ret = unsafe { fork() }.map_err(|e| {
//             let msg = format!("fork error: {}", e);
//             error!("{}", msg);

//             BuckyError::from(msg)
//         })?;

//         match ret {
//             ForkResult::Parent { child } => {
//                 info!("fork child as daemon success: {}", child);

//                 match waitpid(child, None) {
//                     Ok(status) => {
//                         info!("fork child exit: {} {:?}", child, status);
//                         if let WaitStatus::Exited(_pid, code) = status {
//                             if code == 0 {
//                                 return Ok(());
//                             }
//                         }

//                         let msg = format!("fork child but wait error: {}, {:?}", child, status);
//                         error!("{}", msg);

//                         Err(BuckyError::from(msg))
//                     }
//                     Err(e) => {
//                         let msg = format!("fork child wait error: {} {}", child, e);
//                         error!("{}", msg);

//                         Err(BuckyError::from(msg))
//                     }
//                 }
//             }

//             ForkResult::Child => {
//                 match setsid() {
//                     Ok(sid) => {
//                         info!("new sid: {}", sid);
//                     }
//                     Err(e) => {
//                         error!("setsid error: {}", e);
//                         exit(1);
//                     }
//                 }

//                 let mut parts: Vec<&str> = crate::ProcessUtil::parse_cmd(cmd_line);
//                 assert!(parts.len() > 0);

//                 let mut cmd = Command::new(parts[0]);
//                 if parts.len() > 1 {
//                     parts.remove(0);
//                     cmd.args(&parts);
//                 }

//                 let code = match cmd.spawn() {
//                     Ok(_) => {
//                         info!("spawn daemon success!");
//                         0
//                     }
//                     Err(err) => {
//                         error!("spawn daemon error: {}", err);
//                         1
//                     }
//                 };

//                 exit(code);
//             }
//         }
//     }
// }

// #[cfg(windows)]
pub mod daemon {
    use super::ProcessAction;

    pub(super) fn launch_as_daemon(_service_name: &str) -> ProcessAction {
        // let mut cmd = Command::new(service_name);

        // pub const DETACHED_PROCESS: u32 = 0x00000008;
        // pub const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        // pub const CREATE_NO_WINDOW: u32 = 0x08000000;

        // let flags = 0/* DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP |  CREATE_NO_WINDOW*/;
        // cmd.creation_flags(flags)
        //    .stdin(Stdio::null())
        //    .stdout(Stdio::null())
        //    .stderr(Stdio::null());

        // match cmd.spawn() {
        // Ok(_) => {
        ProcessAction::Start
        // }
        // Err(err) => {
        //     ProcessAction::Exit(format!("spawn as daemon error: {} {}", service_name, err))
        // }
    }

/*
        let mut parts: Vec<&str> = crate::ProcessUtil::parse_cmd(cmd_line);
        assert!(parts.len() > 0);

        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            parts.remove(0);
            cmd.args(&parts);
        }

        crate::ProcessUtil::detach(&mut cmd);

        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        match cmd.spawn() {
            Ok(_) => {
                info!("spawn as daemon success: {}", cmd_line);

                Ok(())
            }
            Err(err) => {
                let msg = format!("spawn as daemon error: {} {}", cmd_line, err);
                error!("{}", msg);

                Err(BuckyError::from(msg))
            }
        }
*/
        // Ok(())
    // }
}
