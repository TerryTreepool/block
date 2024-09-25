
use common::{ProcessCommandBuild, ProcessAction, RuntimeProcessTrait};

use log::info;

use hci_service_e::{SERVICE_NAME, process::Process};

#[async_std::main]
async fn main() {
    let process = match ProcessCommandBuild::new(SERVICE_NAME, true)
                                            .launch(Box::new(Process::new(SERVICE_NAME)))
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

