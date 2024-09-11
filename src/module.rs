use std::error::Error;

use libloading::Library;

#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;

use torustiq_common::ffi::{
    types::{
        functions::{ModuleFreeRecordFn, ModuleGetInfoFn, ModuleInitFn, ModuleProcessRecordFn, ModuleStepConfigureFn, ModuleStepSetParamFn},
        module::{ModuleInfo as FfiModuleInfo, ModuleProcessRecordFnResult, ModuleStepConfigureArgs, ModuleStepConfigureFnResult, Record}},
    utils::strings::{cchar_const_deallocate, cchar_to_string, string_to_cchar}};

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

pub struct Module {
    /// A library handle needs to be stored in order to keep the imported functions available
    _lib: Library,
    pub module_info: ModuleInfo,

    init_ptr: RawSymbol<ModuleInitFn>,
    step_init_ptr: RawSymbol<ModuleStepConfigureFn>,
    step_set_param_ptr: RawSymbol<ModuleStepSetParamFn>,
    pub process_record_ptr: RawSymbol<ModuleProcessRecordFn>,
    pub free_record_ptr: RawSymbol<ModuleFreeRecordFn>,
}

pub struct ModuleInfo {
    pub id: String,
    pub name: String,
}

impl From<FfiModuleInfo> for ModuleInfo {
    fn from(value: FfiModuleInfo) -> Self {
        ModuleInfo {
            id: cchar_to_string(value.id),
            name: cchar_to_string(value.name),
        }
    }
}

impl Module {
    pub fn from_library(lib: Library) -> Result<Module, Box<dyn Error>> {
        let loader = RawPointerLoader::new(&lib);
        let module_info: ModuleInfo = {
            let torustiq_module_get_info: RawSymbol<ModuleGetInfoFn> = loader.load(b"torustiq_module_get_info")?;
            torustiq_module_get_info()
        }.into();

        let init_ptr: RawSymbol<ModuleInitFn> = loader.load(b"torustiq_module_init")?;
        let step_init_ptr: RawSymbol<ModuleStepConfigureFn> = loader.load(b"torustiq_module_step_configure")?;
        let step_set_param_ptr: RawSymbol<ModuleStepSetParamFn> = loader.load(b"torustiq_module_step_set_param")?;
        let process_record_ptr: RawSymbol<ModuleProcessRecordFn> = loader.load(b"torustiq_module_process_record")?;
        let free_record_ptr: RawSymbol<ModuleFreeRecordFn> = loader.load(b"torustiq_module_free_record")?;

        Ok(Module {
            _lib: lib,
            module_info,

            init_ptr,
            step_init_ptr,
            step_set_param_ptr,
            process_record_ptr,
            free_record_ptr,
        })
    }

    pub fn get_id(&self) -> String {
        self.module_info.id.clone()
    }

    pub fn init(&self) {
        (self.init_ptr)()
    }

    pub fn configure_step(&self, args: ModuleStepConfigureArgs) -> Result<(), String> {
        let step_handle = args.step_handle;
        match (self.step_init_ptr)(args) {
            ModuleStepConfigureFnResult::Ok => Ok(()),
            ModuleStepConfigureFnResult::ErrorKindNotSupported => Err(String::from("The module cannot be used in this step")),
            ModuleStepConfigureFnResult::ErrorMultipleStepsNotSupported(existing_step_handle) =>
                Err(format!("Cannot configure step with handle {}: \
                            the module supports only one instance of steps \
                            and is already registered in step {}", step_handle, existing_step_handle)),
            ModuleStepConfigureFnResult::ErrorMisc(e) => Err(cchar_to_string(e)),
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
        (self.process_record_ptr)(input, i)
    }

    pub fn free_record(&self, r: Record) {
        (self.free_record_ptr)(r);
    }
}