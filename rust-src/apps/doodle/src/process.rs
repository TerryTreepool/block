
use std::{sync::Arc, path::PathBuf, io::Write,};

use chrono::{DateTime, Local, Timelike, NaiveTime};
use near_base::{NearResult, NearError, ErrorCode};
use near_core::get_service_path;

use common::RuntimeProcessTrait;
use serial::SerialPort;

#[derive(Clone)]
pub(super) struct Config {
    #[allow(unused)]
    work_path: PathBuf,
}

struct ProcessComponents {
}

struct ProcessImpl {
    service_name: String,
    config: Config,

    components: Option<ProcessComponents>,
}

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

unsafe impl Send for Process {}
unsafe impl Sync for Process {}

impl Process {
    pub fn new(service_name: &str) -> NearResult<Box<Self>> {
        let config = Config {
            work_path: get_service_path(service_name),
        };

        let ret = Self(Arc::new(ProcessImpl{
            service_name: service_name.to_owned(),
            config: config.clone(),
            components: None,
        }));

        let mut_ret = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ProcessImpl) };
        mut_ret.components = Some(ProcessComponents {
        });

        Ok(Box::new(ret))
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn service_name(&self) -> &str {
        &self.0.service_name
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn config(&self) -> &Config {
        &self.0.config
    }

}

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {
    async fn run(&self) -> NearResult<()> {

        main_run().await?;

        Ok(())
    }

    fn quit(&self) {
        
    }
}

async fn main_run() -> NearResult<()> {
    println!("open com2");
    let mut com_port = 
        serial::open("/dev/ttymxc1").map_err(| e | {
            let error_string = format!("failed open 'COM2' with err: {e}");
            NearError::new(ErrorCode::NEAR_ERROR_FATAL, error_string)
        })?;

    println!("setting com2");
    com_port.reconfigure(&| settings | {
        settings.set_baud_rate(serial::Baud9600)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })
    .map_err(| e | {
        let error_string = format!("failed setting 'COM2' with err: {e}");
        NearError::new(ErrorCode::NEAR_ERROR_FATAL, error_string)
    })?;

    enum NearTime {
        AM(NaiveTime),
        PM(NaiveTime),
    }

    impl From<DateTime<Local>> for NearTime {
        fn from(value: DateTime<Local>) -> Self {
            let (f, hour) = value.hour12();

            if f {
                Self::PM(NaiveTime::from_hms_opt(hour, value.minute(), value.second()).unwrap())
            } else {
                Self::AM(NaiveTime::from_hms_opt(hour, value.minute(), value.second()).unwrap())
            }
        }
    }

    impl std::fmt::Display for NearTime {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::AM(v) => write!(f, "01{:02X}{:02X}{:02X}", v.hour(), v.minute(), v.second()),
                Self::PM(v) => write!(f, "02{:02X}{:02X}{:02X}", v.hour(), v.minute(), v.second()),
            }
        }
    }

    println!("start");
    async_std::task::spawn(async move {
        loop {

            let now_time: NearTime = chrono::Local::now().into();

            let text = format!(r#"0xAA{}55"#, now_time);

            println!("output: {text}");
            let _ = com_port.write(text.as_bytes());

            let _ = async_std::future::timeout(
                        std::time::Duration::from_secs(1), 
                        async_std::future::pending::<()>()
                    ).await;
        }
    });

    Ok(())
}
