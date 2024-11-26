/// Pipeline step modules

use std::error::Error;

use libloading::Library;
#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;

use torustiq_common::ffi::{
    types::{
        functions as fn_defs,
        module::{
            ModuleProcessRecordFnResult, ModuleStepConfigureArgs,
            ModuleStepConfigureFnResult, ModuleStepStartFnResult, Record
        },
    },
    utils::strings::{cchar_const_deallocate, cchar_to_string, string_to_cchar}
};

use crate::modules::{BaseModule, ModuleInfo};

/// A helper structrure for loading raw symbols
struct RawPointerLoader<'a> {
    lib: &'a Library
}

impl<'a> RawPointerLoader<'a> {
    fn new(lib: &'a Library) -> Self {
        RawPointerLoader { lib }
    }

    pub fn load<T>(&self, symbol: &[u8]) -> Result<RawSymbol<T>, Box<dyn Error>> {
        let s = unsafe { self.lib.get::<T>(symbol) };
        let s = match s {
            Ok(s) => s,
            Err(e) => return Err(Box::new(e)),
        };
        let s = unsafe {s.into_raw()};
        Ok(s)
    }
}

/// A pipeline step module.
/// This kind of modules is involved directly in data processing:
/// it produces, transforms, or writes the data to destination
pub struct StepModule {
    base: BaseModule,

    step_configure_ptr: RawSymbol<fn_defs::ModuleStepConfigureFn>,
    step_set_param_ptr: RawSymbol<fn_defs::ModuleStepSetParamFn>,
    pub step_shutdown_ptr: RawSymbol<fn_defs::ModuleStepShutdownFn>,
    step_start_ptr: RawSymbol<fn_defs::ModuleStepStartFn>,
    pub step_process_record_ptr: RawSymbol<fn_defs::ModuleProcessRecordFn>,
    pub free_char_ptr: RawSymbol<fn_defs::ModuleFreeCharPtrFn>,
    pub free_record_ptr: RawSymbol<fn_defs::ModuleFreeRecordFn>,
}

impl TryFrom<Library> for StepModule {
    type Error = Box<dyn Error>;

    fn try_from(value: Library) -> Result<StepModule, Self::Error> {
        let loader = RawPointerLoader::new(&value);
        let module_info: ModuleInfo = {
            let torustiq_module_get_info: RawSymbol<fn_defs::ModuleGetInfoFn> = loader.load(b"torustiq_module_get_info")?;
            torustiq_module_get_info()
        }.into();

        Ok(StepModule {
            step_configure_ptr: loader.load(b"torustiq_module_step_configure")?,
            step_set_param_ptr: loader.load(b"torustiq_module_step_set_param")?,
            step_shutdown_ptr: loader.load(b"torustiq_module_step_shutdown")?,
            step_start_ptr: loader.load(b"torustiq_module_step_start")?,
            step_process_record_ptr: loader.load(b"torustiq_module_step_process_record")?,
            free_char_ptr: loader.load(b"torustiq_module_free_char_ptr")?,
            free_record_ptr: loader.load(b"torustiq_module_free_record")?,
            
            base: BaseModule {
                init_ptr: loader.load(b"torustiq_module_init")?,

                module_info,
                _lib: value,
            },
        })
    }
}

impl StepModule {
    pub fn get_id(&self) -> String {
        self.base.module_info.id.clone()
    }

    pub fn get_info(&self) -> &ModuleInfo {
        self.base.get_info()
    }

    pub fn init(&self) {
        self.base.init();
    }

    pub fn configure_step(&self, args: ModuleStepConfigureArgs) -> Result<(), String> {
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
                (self.free_char_ptr)(e);
                Err(err_string)
            },
        }
    }

    pub fn start_step(&self, step_handle: usize) -> Result<(), String> {
        match (self.step_start_ptr)(usize::try_into(step_handle).unwrap()) {
            ModuleStepStartFnResult::Ok => Ok(()),
            ModuleStepStartFnResult::ErrorMisc(e) => {
                let err_string = cchar_to_string(e.clone());
                (self.free_char_ptr)(e);
                Err(err_string)
            },
        }
    }

    pub fn set_step_param<S: Into<String>>(&self, handle: usize, k: S, v: S) {
        let k = string_to_cchar(k);
        let v = string_to_cchar(v);
        (self.step_set_param_ptr)(usize::try_into(handle).unwrap(), k, v);

        cchar_const_deallocate(k);
        cchar_const_deallocate(v);
    }

    pub fn process_record(&self, input: Record, step_handle: usize) -> ModuleProcessRecordFnResult {
        let i = usize::try_into(step_handle).unwrap();
        (self.step_process_record_ptr)(input, i)
    }

    pub fn free_record(&self, r: Record) {
        (self.free_record_ptr)(r);
    }

    pub fn shutdown(&self, step_handle: usize) {
        (self.step_shutdown_ptr)(usize::try_into(step_handle).unwrap());
    }
}