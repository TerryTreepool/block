use near_base::DeviceObject;


pub mod probe;


pub struct Configure {
    recv_timeout: std::time::Duration,
}

impl std::default::Default for Configure {
    fn default() -> Self {
        Self {
            recv_timeout: std::time::Duration::from_secs(5),
        }
    }
}

#[derive(Clone)]
pub struct ProbeResult{
    pub desc_list: Vec<DeviceObject>,
}
