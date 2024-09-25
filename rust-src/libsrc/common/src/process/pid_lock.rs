
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use fs2::FileExt;

use near_base::{NearResult, NearError, ErrorCode};

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[cfg(windows)]
use super::win_process::Process;

enum PidlockState {
    New,
    Acquired(File),
    Released,
}

#[allow(unused)]
fn getpid() -> u32 {
    unsafe { libc::getpid() as u32 }
}

fn process_exists(pid: i32) -> bool {
    // From the POSIX standard: If sig is 0 (the null signal), error checking
    // is performed but no signal is actually sent. The null signal can be
    // used to check the validity of pid.
    #[cfg(not(windows))]
    unsafe {
        let result = libc::kill(pid, 0);
        result == 0
    }

    #[cfg(windows)]
    {
        match Process::open(pid as u32) {
            Ok(_) => true,
            Err(_e) => false,
        }
    }
}

fn kill_process(pid: i32) -> bool {
    #[cfg(not(windows))]
    {
        nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), nix::sys::signal::SIGTERM).is_ok()
    }

    #[cfg(windows)]
    {
        let ret = Process::open(pid as u32);
        if let Err(e) = ret {
            println!("open process for kill failed! pid={}, err={}", pid, e);
            return false;
        }

        let proc = ret.unwrap();
        match proc.kill() {
            Ok(_) => true,
            Err(_e) => false,
        }
    }
}

pub struct PidLock {
    pid_state: PidlockState,
    pid_path: PathBuf,
}

impl PidLock {
    pub fn new(pid_path: PathBuf) -> Self {
        PidLock {
            pid_path: pid_path,
            pid_state: PidlockState::New,
        }
    }

    #[allow(unused)]
    pub fn check(&mut self) -> i32 {
        self.check_stale()
    }

    // // 检查当前pid文件里面的path是否和fid匹配
    // pub fn check_fid(&self, fid: &str) -> NearResult<bool> {
    //     let fid = fid.to_owned();

    //     match fs::OpenOptions::new().read(true).open(self.path.as_path()) {
    //         Ok(mut file) => {
    //             let mut contents = String::new();
    //             if let Err(e) = file.read_to_string(&mut contents) {
    //                 error!("read file error: {}", e);
    //                 return Err(e.into());
    //             }

    //             let info: Vec<&str> = contents.trim().split("|").collect();

    //             if info.len() < 2 {
    //                 let msg = format!("invalid pid file format: {}", contents);
    //                 error!("{}", msg);
    //                 Err(BuckyError::new(BuckyErrorCode::InvalidFormat, msg))
    //             } else {
    //                 let path_str = info[1].to_owned();
    //                 match path_str.find(&fid) {
    //                     Some(_) => {
    //                         debug!("fid found in exe path! fid={}, path={}", fid, path_str);
    //                         Ok(true)
    //                     }
    //                     None => {
    //                         warn!("fid not found in exe path! fid={}, path={}", fid, path_str);
    //                         Ok(false)
    //                     }
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             error!("open pid file error! file={} ,err={}", self.path.display(), e);
    //             Err(e.into())
    //         }
    //     }
    // }

    fn check_stale(&self) -> i32 {

        std::fs::read_to_string(self.pid_path.as_path())
            .map(| txt | {
                txt.parse::<i32>()
                    .map(| v | if process_exists(v) { v } else { -1 })
                    .unwrap_or(-1)
            })
            .map_err(| e | {
                println!("failed read {} with err: {e}", self.pid_path.display());
            })
            .unwrap_or(-1)

    }

    pub fn kill(&self) -> i32 {

        let pid = self.check_stale();

        if pid > 0 {
            println!("will kill process: {}, pid: {}", self.pid_path.display(), pid);
            kill_process(pid);
        }

        pid
    }

    pub fn acquire(&mut self) -> NearResult<()> {
        match self.pid_state {
            PidlockState::New => { Ok(()) }
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "invalid state"))
            }
        }?;

        if self.pid_path.exists() {
            if let Err(e) = fs::remove_file(self.pid_path.as_path()) {
                println!("remove old pid file error: {} {}", self.pid_path.display(), e);
            }
        }

        let mut f = {
            #[cfg(windows)]
            {
                fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .share_mode(winapi::um::winnt::FILE_SHARE_READ)
                    .open(self.pid_path.as_path())
            }
            #[cfg(not(windows))]
            {
                fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(self.pid_path.as_path())
            }
        }
        .map_err(| e | {
            NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, 
                           format!("acquire pid lock failed! file={}, err={e}", self.pid_path.display()))
        })?;

        f.write_all(format!("{}", std::process::id().to_string()).as_bytes()).unwrap();

        if let Err(e) = f.lock_shared() {
            Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, 
                               format!("lock pid file error! file={}, err={e}", self.pid_path.display())))
        } else {
            Ok(())
        }?;

        self.pid_state = PidlockState::Acquired(f);

        Ok(())
    }

    #[allow(unused)]
    pub fn release(&mut self) -> NearResult<()> {
        match &self.pid_state {
            PidlockState::Acquired(f) => {
                fs::remove_file(self.pid_path.as_path()).unwrap();
                self.pid_state = PidlockState::Released;
                Ok(())
            }
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "invalid state"))
            }
        }
    }
}
