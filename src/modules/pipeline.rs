/// Pipeline step modules

#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;

use torustiq_common::ffi::{
    types::{
        functions as fn_defs,
        module as module_types,
    },
    utils::strings::cchar_to_string,
};

use crate::{
    callbacks,
    modules::{BaseModule, LibInfo}
};

/// A pipeline step module.
/// This kind of modules is involved directly in data processing:
/// it produces, transforms, or writes the data to destination
#[derive(Clone)]
pub struct PipelineModule {
    pub base: BaseModule,

    pub init_ptr: RawSymbol<fn_defs::LibPipelineInitFn>,
    pub configure_ptr: RawSymbol<fn_defs::ModulePipelineConfigureFn>,
    pub process_record_ptr: RawSymbol<fn_defs::ModulePipelineProcessRecordFn>,
    pub free_record_ptr: RawSymbol<fn_defs::ModuleFreeRecordFn>,
}

impl PipelineModule {
    pub fn init(&self) {
        (self.init_ptr)(module_types::LibPipelineInitArgs {
            common: module_types::LibCommonInitArgs {
                on_step_terminate_cb: callbacks::on_step_terminate_cb,
            },
            on_data_receive_cb: callbacks::on_rcv_cb,
        })
    }

    pub fn get_id(&self) -> String {
        self.base.module_info.id.clone()
    }

    pub fn get_info(&self) -> &LibInfo {
        self.base.get_info()
    }

    pub fn configure(&self, args: module_types::ModulePipelineConfigureArgs) -> Result<(), String> {
        let module_handle = args.module_handle;
        match (self.configure_ptr)(args) {
            module_types::ModulePipelineConfigureFnResult::Ok => Ok(()),
            module_types::ModulePipelineConfigureFnResult::ErrorKindNotSupported => Err(format!("The module cannot be used in step with handle '{}'", module_handle)),
            module_types::ModulePipelineConfigureFnResult::ErrorMultipleStepsNotSupported(existing_module_handle) =>
                Err(format!("Cannot configure step with handle {}: \
                            the module supports only one instance of steps \
                            and is already registered in step {}", module_handle, existing_module_handle)),
                            module_types::ModulePipelineConfigureFnResult::ErrorMisc(e) => {
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

    pub fn process_record(&self, module_handle: usize, input: module_types::Record) -> module_types::ModulePipelineProcessRecordFnResult {
        let i = usize::try_into(module_handle).unwrap();
        (self.process_record_ptr)(i, input)
    }

    pub fn free_record(&self, r: module_types::Record) {
        (self.free_record_ptr)(r);
    }

    pub fn shutdown(&self, module_handle: usize) {
        self.base.shutdown(module_handle);
    }

    pub fn free_c_char(&self, c: *const i8) {
        self.base.free_c_char(c);
    }
}