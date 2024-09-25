
use crate::LogLevel;

#[derive(Clone)]
pub struct LogModuleConfig {
    pub name: String,

    pub level: LogLevel,

    // 是否输出控制台日志
    pub console: LogLevel,

    // 是否输出文件日志
    pub file: bool,

    // 是否使用独立文件
    pub file_name: Option<String>,

    // 单个日志文件的最大大小，字节
    pub file_max_size: u64,

    // 日志文件最大个数，滚动输出
    pub file_max_count: u32,
}

impl LogModuleConfig {

    pub fn new_default(name: &str) -> Self {
        Self {
            name: name.to_owned(),

            level: LogLevel::default(),
            console: LogLevel::default(),
            file: true,
            file_name: Some(name.to_string()),
            file_max_size: 1024 * 1024 * 10,
            file_max_count: 10,
        }
    }

    pub fn set_level(&mut self, level: LogLevel) -> &mut Self {
        self.level = level;
        self
    }

    pub fn set_console(&mut self, level: LogLevel) -> &mut Self {
        self.console = level;
        self
    }

    pub fn set_file(&mut self, file: bool) -> &mut Self {
        self.file = file;
        self
    }

    pub fn set_file_max_size(&mut self, file_max_size: u64) -> &mut Self {
        self.file_max_size = file_max_size;
        self
    }

    pub fn set_file_max_count(&mut self, file_max_count: u32) -> &mut Self {
        self.file_max_count = file_max_count;
        self
    }
}

