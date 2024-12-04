/// Pipeline step modules

#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;

use torustiq_common::ffi::{
    types::{
        functions as fn_defs,
        module::{
            ModuleProcessRecordFnResult, ModulePipelineStepConfigureArgs,
            ModuleStepConfigureFnResult, Record
        },
    },
    utils::strings::cchar_to_string,
};

use crate::modules::{BaseModule, ModuleInfo};

/// A pipeline step module.
/// This kind of modules is involved directly in data processing:
/// it produces, transforms, or writes the data to destination
pub struct PipelineModule {
    pub base: BaseModule,

    pub step_configure_ptr: RawSymbol<fn_defs::ModuleStepConfigureFn>,
    pub step_process_record_ptr: RawSymbol<fn_defs::ModuleProcessRecordFn>,
    pub free_record_ptr: RawSymbol<fn_defs::ModuleFreeRecordFn>,
}

impl PipelineModule {
    pub fn get_id(&self) -> String {
        self.base.module_info.id.clone()
    }

    pub fn get_info(&self) -> &ModuleInfo {
        self.base.get_info()
    }

    pub fn init(&self) {
        self.base.init();
    }

    pub fn configure_step(&self, args: ModulePipelineStepConfigureArgs) -> Result<(), String> {
        let step_handle = args.step_handle;
        match (self.step_configure_ptr)(args) {
            ModuleStepConfigureFnResult::Ok => Ok(()),
            ModuleStepConfigureFnResult::ErrorKindNotSupported => Err(format!("The module cannot be used in step with handle '{}'", step_handle)),
            ModuleStepConfigureFnResult::ErrorMultipleStepsNotSupported(existing_step_handle) =>
                Err(format!("Cannot configure step with handle {}: \
                            the module supports only one instance of steps \
                            and is already registered in step {}", step_handle, existing_step_handle)),
            ModuleStepConfigureFnResult::ErrorMisc(e) => {
                let err_string = cchar_to_string(e.clone());
                self.free_c_char(e);
                Err(err_string)
            },
        }
    }

    pub fn start_step(&self, step_handle: usize) -> Result<(), String> {
        self.base.start_step(step_handle)
    }

    pub fn set_step_param<S: Into<String>>(&self, handle: usize, k: S, v: S) {
        self.base.set_step_param(handle, k, v);
    }

    pub fn process_record(&self, input: Record, step_handle: usize) -> ModuleProcessRecordFnResult {
        let i = usize::try_into(step_handle).unwrap();
        (self.step_process_record_ptr)(input, i)
    }

    pub fn free_record(&self, r: Record) {
        (self.free_record_ptr)(r);
    }

    pub fn shutdown(&self, step_handle: usize) {
        self.base.shutdown(step_handle);
    }

    pub fn free_c_char(&self, c: *const i8) {
        self.base.free_c_char(c);
    }
}