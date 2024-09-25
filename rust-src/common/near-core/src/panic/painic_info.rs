use std::{panic::PanicInfo, thread};

use backtrace::{Backtrace, BacktraceFrame};
use near_base::hash_data;

#[derive(Default)]
pub struct PanicInnerInfo {
    pub msg: String,
    pub msg_with_symbol: String,
    pub hash: String,

}

impl PanicInnerInfo {
    pub fn new(backtrace: Backtrace, info: &PanicInfo) -> Self {
        let backtrace_msg = Self::format_backtrace(&backtrace);
        let msg = Self::format_info(info, &backtrace_msg);

        let backtrace_msg = Self::format_backtrace_with_symbol(&backtrace);
        let msg_with_symbol = Self::format_info(info, &backtrace_msg);

        let hash = Self::calc_hash(&backtrace);

        Self {
            msg,
            msg_with_symbol,
            hash,
        }
    }

    #[allow(unused)]
    fn format_backtrace(backtrace: &Backtrace) -> String {
        let frames: Vec<BacktraceFrame> = backtrace.clone().into();
        let mut values = Vec::new();
        for (i, frame) in frames.into_iter().enumerate() {
            if let Some(mod_addr) = frame.module_base_address() {
                let offset = frame.symbol_address() as isize - mod_addr as isize;
                values.push(format!("{}: {:#018x} {:#018p}", i, offset, mod_addr));
            } else {
                values.push(format!("{}: {:#018p}", i, frame.symbol_address()));
            }
        }

        values.join("\n")
    }

    fn format_info(info: &PanicInfo, backtrace: &str) -> String {
        let thread = thread::current();
        let thread = thread.name().unwrap_or("unnamed");

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };

        let msg = match info.location() {
            Some(location) => {
                format!(
                    "thread '{}' panicked at '{}': {}:{}\n{}",
                    thread,
                    msg,
                    location.file(),
                    location.line(),
                    backtrace,
                )
            }
            None => {
                format!(
                    "thread '{}' panicked at '{}'\n{}",
                    thread,
                    msg,
                    backtrace
                )
            }
        };

        msg
    }

    fn format_backtrace_with_symbol(backtrace: &Backtrace) -> String {
        format!("{:?}", backtrace)
    }

    fn calc_hash(backtrace: &Backtrace) -> String {
        let frames: Vec<BacktraceFrame> = backtrace.clone().into();
        let mut values = Vec::new();
        for (i, frame) in frames.into_iter().enumerate() {
            if let Some(mod_addr) = frame.module_base_address() {
                let offset = frame.symbol_address() as isize - mod_addr as isize;
                values.push(format!("{}:{}", i, offset));
            } else {
                values.push(format!("{}:{:p}", i, frame.symbol_address()));
            }
        }

        let hash = hash_data(values.join("\n").as_bytes());

        hash.to_hex_string()[..32].to_owned()
    }
}
