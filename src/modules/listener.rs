/// Pipeline step modules

#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;

use torustiq_common::ffi::{
    types:: {
        functions as fn_defs,
        module::{
            ModuleListenerConfigureArgs, ModuleListenerConfigureFnResult, StepStartFnResult
        },
    },
    utils::strings::{cchar_const_deallocate, cchar_to_string, string_to_cchar}
};

use crate::modules::{BaseModule, ModuleInfo};

/// An event listener module.
/// Handles the application events, but does not participate in data processing.
pub struct ListenerModule {
    pub base: BaseModule,
    pub configure_ptr: RawSymbol<fn_defs::ModuleListenerConfigureFn>,
}

impl ListenerModule {
    pub fn get_id(&self) -> String {
        self.base.module_info.id.clone()
    }

    pub fn get_info(&self) -> &ModuleInfo {
        self.base.get_info()
    }

    pub fn init(&self) {
        self.base.init();
    }

    pub fn configure(&self, args: ModuleListenerConfigureArgs) -> Result<(), String> {
        match (self.configure_ptr)(args) {
            ModuleListenerConfigureFnResult::Ok => Ok(()),
            ModuleListenerConfigureFnResult::ErrorMisc(e) => {
                let err_string = cchar_to_string(e.clone());
                (self.base.free_char_ptr)(e);
                Err(err_string)
            },
        }
    }

    pub fn start(&self, module_handle: usize) -> Result<(), String> {
        match (self.base.start_ptr)(usize::try_into(module_handle).unwrap()) {
            StepStartFnResult::Ok => Ok(()),
            StepStartFnResult::ErrorMisc(e) => {
                let err_string = cchar_to_string(e.clone());
                (self.base.free_char_ptr)(e);
                Err(err_string)
            },
        }
    }

    pub fn set_param<S: Into<String>>(&self, handle: usize, k: S, v: S) {
        let k = string_to_cchar(k);
        let v = string_to_cchar(v);
        (self.base.set_param_ptr)(usize::try_into(handle).unwrap(), k, v);

        cchar_const_deallocate(k);
        cchar_const_deallocate(v);
    }

    pub fn shutdown(&self, module_handle: usize) {
        (self.base.shutdown_ptr)(usize::try_into(module_handle).unwrap());
    }
}