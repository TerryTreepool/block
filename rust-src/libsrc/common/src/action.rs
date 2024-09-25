
use std::path::PathBuf;

use log::{error, info};
use near_base::{NearResult, ErrorCode, NearError, DeviceObject, ExtentionObject, FileDecoder, PrivateKey, };
use near_core::{LoggerBuilder, get_log_path, panic::PanicBuilder};
use near_transport::ProcessTrait;

use crate::{config::config, 
            RunMode, 
            runtime_stack::RuntimeStack, 
            ProcessAction, 
            RuntimeProcessTrait, 
            core_stack::CoreStack, 
            process::{process_mutex::CURRENT_PROC_LOCK, try_stop_process, try_acquire_process}};

pub const CONSOLE_FLAG: u16 = 0b_00000000_00000001;
pub const DAEMON_FLAG:  u16 = 0b_00000000_00000010;

pub const AUX_FLAG:     u16 = 0b_00000000_00100000;
pub const RUNTIME_FLAG: u16 = 0b_00000000_01000000;
pub const CORE_FLAG:    u16 = 0b_00000000_10000000;

#[derive(Clone, Copy)]
pub enum ProcessActionInner {
    Start(u16),
    Stop,
}

impl ProcessActionInner {

    pub(crate) async fn launch(self, runtime_process_impl: Box<dyn RuntimeProcessTrait>, process_impl: Option<Box<dyn ProcessTrait>>) -> NearResult<ProcessAction> {
        match &self {
            Self::Start(flag) => { ProcessActionInner::start(*flag, runtime_process_impl, process_impl).await }
            Self::Stop => { ProcessActionInner::stop() }
        }
    }

}

impl ProcessActionInner {
    async fn start(flag: u16, runtime_process_impl: Box<dyn RuntimeProcessTrait>, process_impl: Option<Box<dyn ProcessTrait>>) -> NearResult<ProcessAction> {

        let config = config();

        if !*CURRENT_PROC_LOCK {
            println!("{} has exists! ", config.service_name());
            std::process::exit(0);
        }

        {
            if flag & DAEMON_FLAG == DAEMON_FLAG {
                #[cfg(target_os="linux")]
                {
                    use fork::{daemon, Fork};
                    use std::process::Command;
                    use crate::cmd::DAEMON_ARG;

                    if let Ok(Fork::Child) = daemon(true, false) {
                        let args: Vec<String> = std::env::args().collect();
                        // prepair startup child process
                        let mut cmd = Command::new(&args[0]);
                        let daemon_long = DAEMON_ARG.long.map(| long | ["--", long].concat().to_string()).unwrap_or(Default::default());
                        let daemon_short = DAEMON_ARG.short.map(| short | ["-", short.to_string().as_str()].concat().to_string()).unwrap_or(Default::default());
                        for idx in 1..args.len() {
                            let arg = args.get(idx).expect("get cmd arg error!");
                            // prepair starup command-line
                            if arg.eq_ignore_ascii_case(&daemon_long) || arg.eq_ignore_ascii_case(&daemon_short) {
                                continue;
                            }
        
                            cmd.arg(arg);
                        }
                        let _child = cmd.spawn().expect("Child process failed to start.");
                        std::process::exit(0);
                    } else {
                        std::process::exit(0);
                    }
                }

                #[cfg(target_os="windows")]
                {
                    use winapi::um::winuser::ShowWindow;
                    use winapi::um::winuser::SW_HIDE;
 
                    unsafe {
                        let handle = winapi::um::wincon::GetConsoleWindow();
                        ShowWindow(handle, SW_HIDE);
                    }
                }
            }
        }

        #[cfg(target_os="linux")]
        {
            // use signal_hook::flag;

            // flag::register(signal, flag)
            use log::debug;
            use nix::sys::signal;
            use nix::sys::signal::Signal::{SIGKILL, SIGSEGV, SIGSTOP, SIGWINCH, SIGIO, SIGCHLD };
            use nix::sys::signal::SigHandler;

            extern "C" fn core_process_quit(sig: libc::c_int) {
                debug!("core_process quit: {sig}");

                CoreStack::get_instance().quit();

                std::process::exit(0);
            }

            extern "C" fn runtime_process_quit(sig: libc::c_int) {
                debug!("runtime_process quit: {sig}");

                RuntimeStack::get_instance().quit();

                std::process::exit(0);
            }

            extern "C" fn default_process_quit(sig: libc::c_int) {
                debug!("default_process_quit quit: {sig}");
            }

            let action_handler = 
                if flag & CORE_FLAG == CORE_FLAG {
                    SigHandler::Handler(core_process_quit)
                } else if flag & RUNTIME_FLAG == RUNTIME_FLAG {
                    SigHandler::Handler(runtime_process_quit)
                } else {
                    SigHandler::Handler(default_process_quit)
                };
            let _ = unsafe {
                for sig in signal::Signal::iterator() {
                    match sig {
                        SIGKILL | SIGSEGV | SIGSTOP | SIGWINCH | SIGIO | SIGCHLD => { /* ignore signal */ },
                        _ => {
                            let _ = signal::signal(sig, action_handler);
                        }
                    }
                }
            };
    
        }

        try_acquire_process(config.service_name())?;

        // build log
        LoggerBuilder::new(config.service_name(), get_log_path().join(config.service_name()))
            .set_level(config.log_level)
            .set_console(config.log_level)
            .build()?;

        // build panic log
        PanicBuilder::new(config.service_name())
            .exit_on_panic(true)
            .log_to_file(true)
            .build()
            .start();

        let action = 
            if flag & CORE_FLAG > 0 {
                if let RunMode::Core(core, core_private_key) = config.mode {
                    ProcessActionInner::start_core(core, core_private_key, runtime_process_impl, process_impl).await
                } else {
                    let error_string = format!("Failed to startup, it isn't core-mode.");
                    error!("{error_string}");
                    Err(NearError::new(ErrorCode::NEAR_ERROR_STARTUP, error_string))
                }
            } else if flag & RUNTIME_FLAG > 0 {
                if let RunMode::Runtime(core, desc) = config.mode {
                    ProcessActionInner::start_runtime(desc, core, runtime_process_impl).await
                } else {
                    let error_string = format!("Failed to startup, it isn't runtime-mode.");
                    error!("{error_string}");
                    Err(NearError::new(ErrorCode::NEAR_ERROR_STARTUP, error_string))
                }
            } else if flag & AUX_FLAG > 0 {
                if let RunMode::Aux = config.mode {
                    ProcessActionInner::start_aux(runtime_process_impl).await
                } else {
                    let error_string = format!("Failed to startup, it isn't aux-mode.");
                    error!("{error_string}");
                    Err(NearError::new(ErrorCode::NEAR_ERROR_STARTUP, error_string))
                }
            } else {
                let error_string = format!("Failed to startup, it's unknown-mode");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_STARTUP, error_string))
            }?;

        Ok(action)
    }

    async fn start_aux(runtime_process_impl: Box<dyn RuntimeProcessTrait>) -> NearResult<ProcessAction> {
        Ok(ProcessAction::Start(runtime_process_impl))
    }

    async fn start_runtime(desc: PathBuf, core: PathBuf, runtime_process_impl: Box<dyn RuntimeProcessTrait>) -> NearResult<ProcessAction> {
        info!("loading desc from:{}", desc.display());
        info!("loading core from:{}", core.display());

        let core_service = DeviceObject::decode_from_file(&core)?;
        let runtime = ExtentionObject::decode_from_file(&desc)?;
    
        info!("desc-id:{}, core-id:{} startup...", runtime.object_id(), core_service.object_id());    

        let runtime_process = RuntimeStack::open(core_service, runtime, runtime_process_impl).await?;

        Ok(ProcessAction::Start(Box::new(runtime_process) as Box<dyn RuntimeProcessTrait>))
    }

    async fn start_core(
        core: PathBuf, 
        core_private_key: PathBuf, 
        runtime_process_impl: Box<dyn RuntimeProcessTrait>,
        process_impl: Option<Box<dyn ProcessTrait>>
    ) -> NearResult<ProcessAction> {
        let process_impl = 
            process_impl.ok_or_else(|| {
                let error_string = "missing process impl.";
                error!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, error_string)
            })?;

        info!("loading core from:{}", core.display());

        let core_service = DeviceObject::decode_from_file(&core)?;
        let core_service_private_key = PrivateKey::decode_from_file(&core_private_key)?;

        info!("core-id: {} startup...", core_service.object_id());

        let core_process = CoreStack::open(core_service, core_service_private_key, runtime_process_impl, process_impl).await?;

        Ok(ProcessAction::Start(Box::new(core_process) as Box<dyn RuntimeProcessTrait>))
    }

    fn stop() -> NearResult<ProcessAction> {
        let config = config();

        if *CURRENT_PROC_LOCK {
            Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{} not exists!", config.service_name())))
        } else {
            Ok(())
        }?;

        let exit_code = try_stop_process(config.service_name());
        println!("[{exit_code} exit...]");

        Ok(ProcessAction::Stop)
    }

}


