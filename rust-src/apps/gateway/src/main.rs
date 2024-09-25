
use std::sync::RwLock;

use app::App;
use near_base::NearResult;
use near_core::{ProcessCommandBuild, Value, ProcessAction};
use near_util::HTTP_STACK_PORT;

mod app;
mod http_server;
mod p;
mod nds_process;

const SERVICE_NAME: &'static str = "gateway";

#[derive(Clone)]
pub struct Config {
    pub db: String,
    pub port: u16,
}

impl Config {
    pub fn new() -> RwLock<Config> {
        RwLock::new(Self{
            db: String::from("data.db"),
            port: HTTP_STACK_PORT,
        })
    }
}

lazy_static::lazy_static! {
    pub static ref MAIN_CONFIG: RwLock<Config> = Config::new();
}

pub fn get_config() -> Config {
    MAIN_CONFIG.read().unwrap().clone()
}

#[async_std::main]
async fn main() {
    let action = {
        #[cfg(debug_assertions)]
        {
            ProcessCommandBuild::new(SERVICE_NAME)
                .append("port", (Some('p'), "port", true))
                .launch(| name, value| {
                    match value {
                        Value::Value(v) => {
                            if name == "port" {
                                MAIN_CONFIG.write().unwrap().port = v.parse::<u16>().unwrap_or(HTTP_STACK_PORT);
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
    App::new(get_config()).await?
        .start()
        .await?;

    async_std::task::block_on(async_std::future::pending::<()>());

    Ok(())
}
