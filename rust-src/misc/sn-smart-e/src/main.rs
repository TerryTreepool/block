
use common::{ProcessCommandBuild, ProcessAction};
use sn_smart_e::process::Process;

const SERVICE_NAME: &str = "sn-smart";

#[async_std::main]
async fn main() {
    let process = 
        match ProcessCommandBuild::with_runtime()
                .name(SERVICE_NAME)
                .launch(Process::new(SERVICE_NAME).await.expect("failed create main process"), None)
                .await {
        Ok(process) => {
            if let ProcessAction::Start(process) = process {
                process
            } else {
                return;
            }
        }
        Err(err) => { panic!("{err}") }
    };

    if let Err(err) = process.run().await {
        panic!("{err}")
    }
}
