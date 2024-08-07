use std::error::Error;

use libloading::{Library, Symbol};

#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;

use torustiq_common::ffi::{
    types::{
        functions::{ModuleGetInfoFn, ModuleInitFn, ModuleProcessRecordFn, ModuleStepInitFn, ModuleStepSetParamFn},
        module::{ModuleInfo as FfiModuleInfo, ModuleStepInitFnResult, ModuleProcessRecordFnResult, ModuleStepInitArgs, Record}},
    utils::strings::{cchar_to_string, string_to_cchar}};

pub struct Module {
    /// A library handle needs to be stored in order to keep the imported functions available
    _lib: Library,
    pub module_info: ModuleInfo,

    init_ptr: RawSymbol<ModuleInitFn>,
    step_init_ptr: RawSymbol<ModuleStepInitFn>,
    step_set_param_ptr: RawSymbol<ModuleStepSetParamFn>,
    pub process_record_ptr: RawSymbol<ModuleProcessRecordFn>,
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
        let module = unsafe {
            let module_info: ModuleInfo = {
                let torustiq_module_get_info: Symbol<ModuleGetInfoFn> = lib.get(b"torustiq_module_get_info")?;
                torustiq_module_get_info()
            }.into();

            let init_ptr = lib.get::<ModuleInitFn>(b"torustiq_module_init")?.into_raw();
            let step_init_ptr = lib.get::<ModuleStepInitFn>(b"torustiq_module_step_init")?.into_raw();
            let step_set_param_ptr = lib.get::<ModuleStepSetParamFn>(b"torustiq_module_step_set_param")?.into_raw();
            let process_record_ptr = lib.get::<ModuleProcessRecordFn>(b"torustiq_module_process_record")?.into_raw();

            Module {
                _lib: lib,
                module_info,

                init_ptr,
                step_init_ptr,
                step_set_param_ptr,
                process_record_ptr,
            }
        };

        Ok(module)
    }

    pub fn get_id(&self) -> String {
        self.module_info.id.clone()
    }

    pub fn init(&self) {
        (self.init_ptr)()
    }

    pub fn init_step(&self, args: ModuleStepInitArgs) -> Result<(), String> {
        match (self.step_init_ptr)(args) {
            ModuleStepInitFnResult::Ok => Ok(()),
            ModuleStepInitFnResult::ErrorKindNotSupported => Err(String::from("The module cannot be used in this step")),
            ModuleStepInitFnResult::ErrorMisc(e) => Err(cchar_to_string(e)),
        }
    }

    pub fn set_step_param<S: Into<String>>(&self, handle: usize, k: S, v: S) {
        (self.step_set_param_ptr)(usize::try_into(handle).unwrap(), string_to_cchar(k), string_to_cchar(v));
    }

    pub fn process_record(&self, input: Record, step_handle: usize) -> ModuleProcessRecordFnResult {
        let i = usize::try_into(step_handle).unwrap();
        (self.process_record_ptr)(input, i)
    }
}