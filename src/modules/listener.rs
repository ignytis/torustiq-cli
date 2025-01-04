/// Pipeline step modules

#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;

use torustiq_common::ffi::{
    types:: {
        functions as fn_defs,
        module as module_types,
    },
    utils::strings::{cchar_const_deallocate, cchar_to_string, string_to_cchar}
};

use crate::{
    callbacks,
    modules::{BaseModule, LibInfo}
};

/// An event listener module.
/// Handles the application events, but does not participate in data processing.
pub struct ListenerModule {
    pub base: BaseModule,

    pub configure_ptr: RawSymbol<fn_defs::ModuleListenerConfigureFn>,

    pub init_ptr: RawSymbol<fn_defs::LibListenerInitFn>,
    /// A pointer to message receive handler
    pub record_rcv_ptr: RawSymbol<fn_defs::ModuleListenerRecordRcvFn>,
    /// A pointer to message send handler (successful)
    pub record_send_success_ptr: RawSymbol<fn_defs::ModuleListenerRecordSendSuccessFn>,
    /// A pointer to message send handler (failure)
    pub record_send_failure_ptr: RawSymbol<fn_defs::ModuleListenerRecordSendFailureFn>,
}

impl ListenerModule {
    pub fn init(&self) {
        (self.init_ptr)(module_types::LibListenerInitArgs {
            common: module_types::LibCommonInitArgs {
                on_step_terminate_cb: callbacks::on_step_terminate_cb,
            },
        })
    }

    pub fn get_id(&self) -> String {
        self.base.module_info.id.clone()
    }

    pub fn get_info(&self) -> &LibInfo {
        self.base.get_info()
    }

    pub fn configure(&self, args: module_types::ModuleListenerConfigureArgs) -> Result<(), String> {
        match (self.configure_ptr)(args) {
            module_types::ModuleListenerConfigureFnResult::Ok => Ok(()),
            module_types::ModuleListenerConfigureFnResult::ErrorMisc(e) => {
                let err_string = cchar_to_string(e.clone());
                (self.base.free_char_ptr)(e);
                Err(err_string)
            },
        }
    }

    pub fn start(&self, module_handle: usize) -> Result<(), String> {
        match (self.base.start_ptr)(usize::try_into(module_handle).unwrap()) {
            module_types::StepStartFnResult::Ok => Ok(()),
            module_types::StepStartFnResult::ErrorMisc(e) => {
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