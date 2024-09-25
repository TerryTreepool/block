
pub mod search_task;
pub mod get_task_result;
pub mod add_thing_task;
pub mod crud_thing_task;
pub mod ctrl_thing_task;
pub mod query_all_thing_task;
pub mod schedule;

#[derive(Default, Clone)]
pub struct Config {
    pub ctrl_config: ctrl_thing_task::Config,
    pub query_task_config: query_all_thing_task::QueryTaskConfig,
}
