/// Pipeline step modules

#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;

use torustiq_common::ffi::{
    types::{
        functions as fn_defs,
        module::{
            ModulePipelineProcessRecordFnResult, ModulePipelineConfigureArgs,
            ModulePipelineConfigureFnResult, Record
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

    pub configure_ptr: RawSymbol<fn_defs::ModulePipelineConfigureFn>,
    pub process_record_ptr: RawSymbol<fn_defs::ModulePipelineProcessRecordFn>,
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

    pub fn configure(&self, args: ModulePipelineConfigureArgs) -> Result<(), String> {
        let module_handle = args.module_handle;
        match (self.configure_ptr)(args) {
            ModulePipelineConfigureFnResult::Ok => Ok(()),
            ModulePipelineConfigureFnResult::ErrorKindNotSupported => Err(format!("The module cannot be used in step with handle '{}'", module_handle)),
            ModulePipelineConfigureFnResult::ErrorMultipleStepsNotSupported(existing_module_handle) =>
                Err(format!("Cannot configure step with handle {}: \
                            the module supports only one instance of steps \
                            and is already registered in step {}", module_handle, existing_module_handle)),
            ModulePipelineConfigureFnResult::ErrorMisc(e) => {
                let err_string = cchar_to_string(e.clone());
                self.free_c_char(e);
                Err(err_string)
            },
        }
    }

    pub fn start(&self, module_handle: usize) -> Result<(), String> {
        self.base.start(module_handle)
    }

    pub fn set_param<S: Into<String>>(&self, handle: usize, k: S, v: S) {
        self.base.set_param(handle, k, v);
    }

    pub fn process_record(&self, input: Record, module_handle: usize) -> ModulePipelineProcessRecordFnResult {
        let i = usize::try_into(module_handle).unwrap();
        (self.process_record_ptr)(input, i)
    }

    pub fn free_record(&self, r: Record) {
        (self.free_record_ptr)(r);
    }

    pub fn shutdown(&self, module_handle: usize) {
        self.base.shutdown(module_handle);
    }

    pub fn free_c_char(&self, c: *const i8) {
        self.base.free_c_char(c);
    }
}