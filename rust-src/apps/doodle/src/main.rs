
pub mod process;

use common::{ProcessCommandBuild, ProcessAction};

use crate::process::Process;

static SERVICE_NAME: &'static str = "doodle";

#[async_std::main]
async fn main() {
    let process = 
        match ProcessCommandBuild::with_aux()
                .name(SERVICE_NAME)
                .launch(Process::new(SERVICE_NAME).expect("failed create main process"), None)
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

