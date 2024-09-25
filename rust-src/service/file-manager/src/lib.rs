
mod module;

use base::ModuleTrait;

use module::Module;
use near_transport::Stack;

#[no_mangle]
pub unsafe extern "C" fn create_extention_module(stack: *const Stack) -> *const Box<dyn ModuleTrait> {
    let module = Module::new((*stack).clone());

    &module.clone_as_module() as *const Box<dyn ModuleTrait>
}

