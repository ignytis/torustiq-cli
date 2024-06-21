use std::error::Error;

use libloading::{Library, Symbol};

#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;

use torustiq_common::ffi::{
    types::{
        functions::{ModuleGetInfoFn, ModuleInitStepFn, ModuleProcessRecordFn},
        module::{IoKind, ModuleInfo as FfiModuleInfo, ModuleInitStepArgs, ModuleProcessRecordFnResult, Record}},
    utils::strings::cchar_to_string};

pub struct Module {
    /// A library handle needs to be stored in order to keep the imported functions available
    _lib: Library,
    pub module_info: ModuleInfo,

    init_step_ptr: RawSymbol<ModuleInitStepFn>,
    pub process_record_ptr: RawSymbol<ModuleProcessRecordFn>,
}

pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub input_kind: IoKind,
    pub output_kind: IoKind,
}

impl From<FfiModuleInfo> for ModuleInfo {
    fn from(value: FfiModuleInfo) -> Self {
        ModuleInfo {
            id: cchar_to_string(value.id),
            name: cchar_to_string(value.name),
            input_kind: value.input_kind,
            output_kind: value.output_kind,
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

            let init_step_ptr = lib.get::<ModuleInitStepFn>(b"torustiq_module_init_step")?.into_raw();
            let process_record_ptr = lib.get::<ModuleProcessRecordFn>(b"torustiq_module_process_record")?.into_raw();

            Module {
                _lib: lib,
                module_info,

                init_step_ptr,
                process_record_ptr,
            }
        };

        Ok(module)
    }

    pub fn init_step(&self, args: ModuleInitStepArgs) {
        (self.init_step_ptr)(args)
    }

    pub fn process_record(&self, input: Record, step_handle: usize) -> ModuleProcessRecordFnResult {
        let i = usize::try_into(step_handle).unwrap();
        (self.process_record_ptr)(input, i)
    }
}