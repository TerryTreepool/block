use std::ptr::null_mut;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::HANDLE;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess, GetExitCodeProcess};
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE};
use winapi::um::minwinbase::STILL_ACTIVE;

pub struct Process {
    pid: DWORD,
    handle: HANDLE,
}

impl Process {
    pub fn open(pid: DWORD) -> Result<Process, String> {
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms684320%28v=vs.85%29.aspx
        let pc = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE, 0, pid) };
        if pc == null_mut() {
            return Err("!OpenProcess".to_string());
        }

        // 打开句柄成功的话，还需要检测进程是否退出
        let mut code: DWORD = 0;
        let ret = unsafe { GetExitCodeProcess(pc, &mut code) };
        if ret == 0 {
            let err = unsafe { GetLastError() };
            let msg = format!("call GetExitCodeProcess error! pid={}, err={}", pid, err);
            println!("{}", msg);
            return Err(msg);
        }

        if code == STILL_ACTIVE {
            Ok(Process{
                pid,
                handle: pc,
            })
        } else {
            println!("process exit already! pid={}, ret={}", pid, ret);
            return Err("process exited already".to_string());
        }
        
    }

    pub fn kill(self) -> Result<(), String> {
        let ret = unsafe { TerminateProcess(self.handle, 1) };
        if ret == 0 {
            let msg = format!("kill process failed! pid={}", self.pid);
            println!("{}", msg);

            Err(msg.to_string())
        } else {
            println!("kill process success! pid={}", self.pid);
            Ok(())
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { winapi::um::handleapi::CloseHandle(self.handle) };
    }
}