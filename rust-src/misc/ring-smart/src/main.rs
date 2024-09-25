
use common::{ProcessCommandBuild, ProcessAction};

use ring_smart::process::Process;

const SERVICE_NAME: &str = "ring-smart";

#[async_std::main]
async fn main() {
    let process = 
        match ProcessCommandBuild::new(SERVICE_NAME, true)
                .launch(Process::new(SERVICE_NAME).await.expect("failed create main process"))
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
