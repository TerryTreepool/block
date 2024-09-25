
use std::path::PathBuf;

use near_base::NearResult;
use near_core::get_service_path;

use super::pid_lock::PidLock;

pub struct ProcessLock {
    #[allow(unused)]
    service_name: String,
    pid_lock: PidLock,
}

impl ProcessLock {
    pub fn new(service_name: &str) -> ProcessLock {
        let pid_folder = get_service_path(service_name);
        let pid_file = pid_folder.join(PathBuf::new().with_file_name(service_name).with_extension("pid"));

        ProcessLock {
            service_name: service_name.to_owned(),
            pid_lock: PidLock::new(pid_file),
        }
    }

    #[allow(unused)]
    pub fn check(&mut self) -> i32 {
        self.pid_lock.check()
    }

    #[allow(unused)]
    pub fn acquire(&mut self) -> NearResult<()> {
        self.pid_lock.acquire()
    }

    #[allow(unused)]
    pub fn release(&mut self) -> NearResult<()> {
        self.pid_lock.release()
    }

    pub fn kill(&self) -> i32 {
        self.pid_lock.kill()
    }
}
