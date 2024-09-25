
use std::path::PathBuf;

use near_base::{NearResult, NearError, ErrorCode};

use common::{ProcessCommandBuild, ProcessAction, };

use log::info;

use hci_service_e::{SERVICE_NAME, process::{Process, Config}};
use near_core::get_data_path;

pub async fn load_from_config(service_name: &str) -> NearResult<Config> {
    let toml_file = PathBuf::new().with_file_name(service_name).with_extension("toml");
    let content = 
        async_std::fs::read_to_string(get_data_path().join(toml_file.as_path()))
            .await
            .map_err(| _ | {
                let error_string = format!("Missing [{}] file, will run with default configuration", toml_file.display());
                println!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
            })?;

        let mut val: toml::Value = 
            toml::from_str(&content).map_err(| e | {
                let error_string = format!("parse [{}] with err: {e}", toml_file.display());
                println!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, error_string)
            })?;

    let load_routines = | val: &mut toml::Value | -> NearResult<hci_service_e::routines::Config> {
        let routines = val.as_table_mut().map(| table | {
            table
        })
        .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found [routines]."))?
        .remove("routines")
        .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found [routines]."))?;

        let ctrl_interval = 
            routines.get("ctrl_thing_task").map(| cfg | {
                let ctrl_config = {
                        std::time::Duration::from_millis(
                             cfg.get("ctrl_interval")
                                .map(| ctrl_interval | ctrl_interval.as_integer().unwrap_or_default())
                                .unwrap_or_default() as u64
                            )
                    };
                ctrl_config
            })
            .unwrap_or(std::time::Duration::ZERO);

        Ok(
            hci_service_e::routines::Config {
                ctrl_config: hci_service_e::routines::ctrl_thing_task::Config {
                    ctrl_interval,
                },
                query_task_config: Default::default(),
            }
        )
    };

    Ok(
        Config {
            routines_config: load_routines(&mut val)?,
            ..Default::default()
        }
    )
}

#[async_std::main]
async fn main() {
    let process = 
        match ProcessCommandBuild::with_runtime()
                .name(SERVICE_NAME)
                .launch(Box::new(Process::new(SERVICE_NAME, load_from_config(SERVICE_NAME).await.ok()).await), None)
                .await {
            Ok(process) => {
                if let ProcessAction::Start(process) = process {
                    process
                } else {
                    info!("exiting...");
                    return;
                }
            }
            Err(err) => {
                panic!("{err}")
            }
        };

    if let Err(err) = process.run().await {
        panic!("{err}")
    }
}

