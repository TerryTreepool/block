use near_base::NearResult;


pub mod process_mutex;
mod process_lock;
pub mod pid_lock;

#[cfg(target_os="windows")]
pub mod win_process;

// 不检查进程锁，直接读取进程pid文件并尝试终止
pub fn try_stop_process(service_name: &str) -> i32 {
    process_lock::ProcessLock::new(service_name).kill()
}

pub fn try_acquire_process(service_name: &str) -> NearResult<()> {
    process_lock::ProcessLock::new(service_name).acquire()
}
