
use common::{ProcessCommandBuild, ProcessAction};
use near_transport::ProcessTrait;
use process::Process;

// mod service;
mod event;
mod process;

const SERVICE_NAME: &str = "core-service";

#[async_std::main]
async fn main() {
    let core_service_p = Process::new(SERVICE_NAME).expect("failed create main process");
    let process_p = core_service_p.clone_as_process();

    let process = 
        match ProcessCommandBuild::with_core()
                .name(SERVICE_NAME)
                .launch(core_service_p, Some(process_p))
                .await {
        Ok(process) => {
            if let ProcessAction::Start(process) = process {
                process
            } else {
                return;
            }
        }
        Err(err) => { println!("{err}"); panic!() }
    };

    if let Err(err) = process.run().await {
        panic!("{err}")
    }
}
