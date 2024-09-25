
use std::path::PathBuf;

use near_util::HTTP_STACK_PORT;

pub const GATEWAY_DATA_DB: &'static str = "gw_data.db";

pub struct Config {
    db: PathBuf,
    port: u16,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            db: Default::default(),
            port: HTTP_STACK_PORT,
        }
    }
}

impl Config {
    pub fn db(mut self, db_path: PathBuf) -> Self {
        self.db = db_path;
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

mod app;
mod p;
mod http_server;
mod nds_process;

pub use app::App;

