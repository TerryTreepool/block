
use std::{path::PathBuf, sync::Arc, panic, fs::create_dir_all, };

use backtrace::Backtrace;
use chrono::{DateTime, Local};

use crate::get_log_path;

use super::painic_info::PanicInnerInfo;

pub struct PanicBuilder {
    service_name: String,

    log_to_file: bool,
    log_dir: PathBuf,

    // bug_reporter: Option<Box<dyn BugReportHandler>>,

    // panic后是否结束进程
    exit_on_panic: bool,
}

impl PanicBuilder {
    pub fn new(service_name: &str) -> Self {
        assert!(!service_name.is_empty());

        Self {
            service_name: service_name.to_owned(),
            log_to_file: true,
            log_dir: get_log_path().join(service_name).join("panic"),
            exit_on_panic: false,
        }
    }

    // panic信息是否输出到日志文件，默认输出
    pub fn log_to_file(mut self, enable: bool) -> Self {
        self.log_to_file = enable;
        self
    }

    // panic输出到的日志目录，默认是{cyfs_root}/log/panic/{product_name}/
    pub fn log_dir(mut self, log_dir: impl Into<PathBuf>) -> Self {
        self.log_dir = log_dir.into();
        self
    }

    // panic后是否结束进程，默认不结束
    pub fn exit_on_panic(mut self, exit: bool) -> Self {
        self.exit_on_panic = exit;
        self
    }

    pub fn build(self) -> PanicManager {
        PanicManager::new(self)
    }
}

struct PanicManagerImpl {
    service_name: String,

    log_to_file: bool,
    log_dir: PathBuf,

    exit_on_panic: bool,

    // on_panic: OnPanicEventManager,

    // // 上报器
    // reporter: Box<dyn BugReportHandler>,
}

#[derive(Clone)]
pub struct PanicManager(Arc<PanicManagerImpl>);

impl PanicManager {
    pub(crate) fn new(builder: PanicBuilder) -> Self {
        Self(Arc::new(PanicManagerImpl{
            service_name: builder.service_name,
            log_to_file: builder.log_to_file,
            log_dir: builder.log_dir,
            exit_on_panic: builder.exit_on_panic
        }))
    }

    pub fn start(&self) {
        let this = self.clone();
        panic::set_hook(Box::new(move |info| {
            let backtrace = Backtrace::new();
            let pinfo = PanicInnerInfo::new(backtrace, info);
            let this = this.clone();

            std::thread::spawn(move || {
                this.on_panic(pinfo);
            });
        }));
    }

}

impl PanicManager {
    fn on_panic(&self, info: PanicInnerInfo) {
        if self.0.log_to_file {
            self.log_to_file(&info);
        }

        println!("will report panic......");
        // let _ = self
        //     .0
        //     .reporter
        //     .notify(&self.0.product_name, &self.0.service_name, &info);

        // // 触发事件
        // let _ = self.0.on_panic.emit(&info);

        if self.0.exit_on_panic {
            crate::log_logger::Logger::flush();

            println!("process will exit on panic......");
            std::thread::sleep(std::time::Duration::from_secs(3));
            println!("process exit on panic......");
            std::process::exit(-1);
        }
    }

    fn log_to_file(&self, info: &PanicInnerInfo) {
        let file_name = format!("{}_panic_{}.log", self.0.service_name, info.hash);

        let now = std::time::SystemTime::now();
        let datetime: DateTime<Local> = now.into();
        let now = datetime.format("%Y_%m_%d %H:%M:%S.%f");

        let content =
        {
            #[cfg(debug_assertions)]
            {
                format!("{}\n{}", now, info.msg_with_symbol)
            }

            #[cfg(not(debug_assertions))]
            {
                format!("{}\n{}", now, info.msg)
            }
        };

        let write_panic = | panic_log_dir: &PathBuf, file_name: &str, content: &str | -> std::io::Result<()> {
            create_dir_all(panic_log_dir.as_path())?;
            std::fs::write(panic_log_dir.join(file_name), content)
        };

        if let Err(e) = write_panic(&self.0.log_dir, file_name.as_str(), &content) {
            println!("write panic log failed! dir={}, {}", file_name, e);
        }
    }

}
