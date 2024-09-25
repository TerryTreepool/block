
mod process;
mod p;
// mod manager;
mod routines;

use common::{ProcessCommandBuild, ProcessAction, };
use log::info;
use process::Process;

use crate::p::SERVICE_NAME;

#[async_std::main]
async fn main() {
    let process = 
        match ProcessCommandBuild::with_runtime()
                    .name(SERVICE_NAME)
                    .launch(Box::new(Process::new(SERVICE_NAME.to_owned())), None).await {
        Ok(process) => {
            if let ProcessAction::Start(process) = process {
                process
            } else {
                info!("exiting...");
                return;
            }
        }
        Err(err) => { panic!("{err}") } 
    };

    if let Err(err) = process.run().await {
        panic!("{err}")
    }
}
