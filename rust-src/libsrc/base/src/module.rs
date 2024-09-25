
use near_transport::ProcessTrait;

pub trait ModuleTrait: ProcessTrait + Send + Sync {
    fn clone_as_module(&self) -> Box<dyn ModuleTrait>;
}
