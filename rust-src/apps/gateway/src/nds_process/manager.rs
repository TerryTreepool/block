
use std::{sync::{Arc, RwLock}, collections::BTreeMap};

use nds::NdsStack;
use near_base::NearResult;

use super::{NdsFileArticle, NdsProcess};

struct NdsManagerImpl {
    stack: NdsStack,
    nds_process: RwLock<BTreeMap<String, NdsProcess>>,
}

pub struct NdsManager(Arc<NdsManagerImpl>);

impl NdsManager {
    pub fn new(stack: NdsStack) -> Self {
        Self(Arc::new(NdsManagerImpl{
            stack,
            nds_process: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn nds_process_of(&self, file_id: &str) -> Option<NdsProcess> {
        self.0.nds_process.read().unwrap()
            .get(file_id)
            .cloned()
    }

    pub fn add_nds_process(&self, file: NdsFileArticle) -> NearResult<()> {
        enum Step {
            Exist,
            NewProcess(NdsProcess),
        }

        let step = {
            let processes = &mut *self.0.nds_process.write().unwrap();
            match processes.get(file.file_id.as_str()) {
                Some(_) => Step::Exist,
                None => {
                    let process = NdsProcess::new(file);
                    processes.insert(process.file_id().to_owned(), process.clone());
                    Step::NewProcess(process)
                }
            }
        };

        match step {
            Step::Exist => {},
            Step::NewProcess(process) => {
                process.run();
            }
        }

        Ok(())
    }
}
