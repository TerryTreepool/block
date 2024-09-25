
mod caches;
mod routines;
mod process;

use common::{ProcessCommandBuild, ProcessAction, };
use log::{error, info};
use process::Process;

use hci_storage_e::SERVICE_NAME;

#[async_std::main]
async fn main() {
    let process = 
        match ProcessCommandBuild::with_runtime()
                    .name(SERVICE_NAME)
                    .launch(Process::new(SERVICE_NAME).await.unwrap(), None).await {
        Ok(process) => {
            if let ProcessAction::Start(process) = process {
                process
            } else {
                info!("exiting...");
                return;
            }
        }
        Err(err) => { error!("{err}"); panic!("{err}") } 
    };

    if let Err(err) = process.run().await {
        panic!("{err}")
    }
}
