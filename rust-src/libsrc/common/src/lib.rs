
mod cmd;
mod config;
mod action;
mod runtime_stack;
mod core_stack;
mod routine_template;
mod topic_runtime;
mod process;

use near_transport::ProcessTrait;
pub use runtime_stack::RuntimeStack;
pub use core_stack::CoreStack;
pub use routine_template::RoutineTemplate;
pub use topic_runtime::manager::{Manager as TopicRouineManager, TopicRoutineCbEventTrait, TopicRoutineOpEventTrait};

use action::{CORE_FLAG, RUNTIME_FLAG, ProcessActionInner, AUX_FLAG};
use clap::command;
use cmd::*;
use config::{action, set_mode, set_log_level, set_service_name, set_action, RunMode, };
use near_base::NearResult;

use process::process_mutex::SERVICE_NAME;

pub enum ProcessAction {
    Start(Box<dyn RuntimeProcessTrait>),
    Stop,
}

#[async_trait::async_trait]
pub trait RuntimeProcessTrait: Send + Sync {
    async fn run(&self) -> NearResult<()>;
    fn quit(&self);
}

pub struct ProcessCommandBuild {
    mode: RunMode,
    process_name: String,
}

impl ProcessCommandBuild {
    pub fn with_core() -> Self {
        Self {
            mode: RunMode::Core(Default::default(), Default::default()),
            process_name: Default::default(),
        }
    }

    pub fn with_runtime() -> Self {
        Self {
            mode: RunMode::Runtime(Default::default(), Default::default()),
            process_name: Default::default(),
        }
    }

    pub fn with_aux() -> Self {
        Self {
            mode: RunMode::Aux,
            process_name: Default::default(),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.process_name = name.to_owned();
        self
    }

}

impl ProcessCommandBuild {
    pub async fn launch(self, runtime_process_impl: Box<dyn RuntimeProcessTrait>, process_impl: Option<Box<dyn ProcessTrait>>) -> NearResult<ProcessAction> {
        SERVICE_NAME.lock().unwrap().init(&self.process_name);

        set_service_name(&self.process_name);

        let matches = 
            command!(self.process_name)
                .arg(LOGLEVEL_ARG.into_arg())
                .args(
                    match &self.mode {
                        RunMode::Core(_, _) => build_core_arg(),
                        RunMode::Runtime(_, _) => build_runtime_arg(),
                        RunMode::Aux => { vec![] },
                        _ => { unreachable!() }
                    }
                )
                .arg(DAEMON_ARG.into_arg())
                .arg(QUIT_ARG.into_arg())
                .get_matches();

        {
            let action = match check_action_with_args(&matches) {
                ProcessActionInner::Start(flag) => {
                    ProcessActionInner::Start(
                        match &self.mode {
                            RunMode::Core(_, _) => flag | CORE_FLAG as u16,
                            RunMode::Runtime(_, _) => flag | RUNTIME_FLAG as u16,
                            RunMode::Aux => flag | AUX_FLAG as u16,
                            _ => unreachable!()
                        }
                    )
                }
                ProcessActionInner::Stop => { ProcessActionInner::Stop }
            };
            set_action(action);
        };

        set_mode(match &self.mode {
            RunMode::Core(_, _) => { check_core_command_with_args(&matches) }
            RunMode::Runtime(_, _) => { check_runtime_command_with_args(&matches) }
            RunMode::Aux => { RunMode::Aux }
            _ => { unreachable!() }
        });

        if let Some(level) = check_log_level_command_with_args(&matches) {
            set_log_level(level);
        }

        action().launch(runtime_process_impl, process_impl).await
    }
}
