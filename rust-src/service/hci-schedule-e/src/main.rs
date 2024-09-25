
use log::info;

use common::{ProcessCommandBuild, ProcessAction};

use hci_schedule_e::{SERVICE_NAME, process::Process};

#[async_std::main]
async fn main() {
    let process = 
        match ProcessCommandBuild::with_runtime()
                    .name(SERVICE_NAME)
                    .launch(Box::new(Process::new(SERVICE_NAME)), None)
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

