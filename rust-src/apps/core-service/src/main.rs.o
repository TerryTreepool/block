
use std::{path::PathBuf,
          sync::RwLock, str::FromStr,
    };
use near_base::{DeviceObject, builder_codec::FileDecoder, NearResult};
use near_core::{ProcessCommandBuild, ProcessAction, Value,
                get_data_path, LoggerBuilder, get_log_path, LogLevel, panic::PanicBuilder, get_root_path,
    };

mod service;
mod event;
use service::*;

const SERVICE_NAME: &str = "core-service";

pub struct Config {
    local_desc: PathBuf,
    log_level: LogLevel,
}

impl Config {
    pub fn new() -> RwLock<Config> {
        RwLock::new(Self{
            local_desc: get_data_path().join(format!("{}.desc", SERVICE_NAME)),
            log_level: LogLevel::default(),
        })
    }
}

lazy_static::lazy_static! {
    pub static ref MAIN_CONFIG: RwLock<Config> = Config::new();
}

#[async_std::main]
async fn main() {
    let action = {
        #[cfg(debug_assertions)]
        {
            ProcessCommandBuild::new(SERVICE_NAME)
                .append("core", (None, "core", true))
                .append("log-level", (None, "log-level", true))
                .launch(| name, value| {
                    match value {
                        Value::Value(v) => {
                            if name == "core" {
                                MAIN_CONFIG.write().unwrap().local_desc = get_data_path().join(format!("{}.desc", v));
                            } else if name == "log-level" {
                                MAIN_CONFIG.write().unwrap().log_level = LogLevel::from_str(v).unwrap_or(LogLevel::default());
                            }
    
                        }
                        _ => {}
                    }
                })
        }

        #[cfg(not(debug_assertions))]
        {
            ProcessAction::Start
        }
    };

    println!("work dir: {}", get_root_path().display());

    match action {
        ProcessAction::Exit(err) => {
            panic!("{} startup failed with error {}", SERVICE_NAME, err);
        }
        ProcessAction::Stop => { /* runtime_stop().await; */ unimplemented!() }
        ProcessAction::Update => { /* runtime_update().await; */ unimplemented!() }
        ProcessAction::Start => { 
            let _ = main_start().await.map_err(| err | {
                        panic!("failed {} startup, by {}", SERVICE_NAME, err);
                    });
        }
    }
}

async fn main_start() -> NearResult<()> {

    LoggerBuilder::new("core", get_log_path().join("core"))
        .set_level(LogLevel::Trace)
        .set_console(LogLevel::default())
        .build()?;

    PanicBuilder::new("core")
        .exit_on_panic(true)
        .log_to_file(true)
        .build()
        .start();

    let local_device = 
        DeviceObject::decode_from_file(MAIN_CONFIG.read().unwrap().local_desc.as_path())?;

    let _ = ServiceStack::open(local_device).await?;

    async_std::task::block_on(async_std::future::pending::<()>());

    Ok(())
}
