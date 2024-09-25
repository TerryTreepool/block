
use std::{str::FromStr,
    };

use async_std;

use log::info;
use near_base::{DeviceObject, builder_codec::FileDecoder, NearResult, ExtentionObject};
use near_core::*;

use near_core::panic::PanicBuilder;

use file_manager_e::{RUNTIME_CONFIG, RuntimeStack};

const LABEL_CORE_DESC: &str = "core";
const LABEL_DESC: &str = "desc";
const LABEL_LOG_LEVEL: &str = "log-level";

#[async_std::main]
async fn main() {
    let action = 
        ProcessCommandBuild::new("gateway-e")
            .author("Near Ltd.")
            .append(LABEL_CORE_DESC, (None, LABEL_CORE_DESC, true))
            .append(LABEL_DESC, (Some('d'), LABEL_DESC, true))
            .append(LABEL_LOG_LEVEL, (None, LABEL_LOG_LEVEL, true))
            .launch(| name, value | {
                match value {
                    Value::Value(v) => {
                        if name == LABEL_CORE_DESC {
                            RUNTIME_CONFIG.set_core_desc(v);
                        }
                        else if name == LABEL_DESC {
                            RUNTIME_CONFIG.set_device_desc(v);
                        } else if name == LABEL_LOG_LEVEL {
                            RUNTIME_CONFIG.set_log_level(LogLevel::from_str(v).unwrap_or(LogLevel::default()));
                        }
                    },
                    Value::Present(_v) => { }
                }
            });

    match action {
        ProcessAction::Exit(err) => {
            panic!("Runtime startup failed with error {}", err);
        }
        ProcessAction::Start => {
            if let Err(err) = runtime_start().await {
                panic!("{}", err)
            }
        }
        ProcessAction::Stop => { runtime_stop().await; }
        ProcessAction::Update => { runtime_update().await; }
    }
    
}

async fn runtime_start() -> NearResult<()> {

    let extention_name = RUNTIME_CONFIG.extention_name();

    LoggerBuilder::new(extention_name.as_str(), get_log_path().join(extention_name.as_str()))
        .set_level(RUNTIME_CONFIG.log_level())
        .set_console(RUNTIME_CONFIG.log_level())
        .build()?;

    PanicBuilder::new(extention_name.as_str())
        .exit_on_panic(true)
        .log_to_file(true)
        .build()
        .start();

    let core_service = DeviceObject::decode_from_file(&RUNTIME_CONFIG.core_desc())?;
    let runtime = ExtentionObject::decode_from_file(&RUNTIME_CONFIG.device_desc())?;

    info!("desc-id:{}, core-id:{} startup...", runtime.object_id(), core_service.object_id());

    let _stack = RuntimeStack::open(core_service, runtime).await?;

    async_std::task::block_on(async_std::future::pending::<()>());

    Ok(())
}

async fn runtime_stop() {

}

async fn runtime_update() {

}
