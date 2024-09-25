
pub mod hci;
pub mod lua;
pub mod tasks;
pub mod process;
pub mod routines;
// pub mod schedule;
pub mod cache;

pub const SERVICE_NAME: &'static str = "hci-service";

pub const MAC_ADDRESS: &'static str = "mac";
pub const SCHEDULE_ID: &'static str = "schedule-id";
pub const TIMES: &'static str       = "times";
pub const SEQ: &'static str         = "seq";
