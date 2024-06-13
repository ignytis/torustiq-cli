use std::error::Error;

use libloading::{Library, Symbol};

use torustiq_common::ffi::{types::{functions::ModuleGetInfoFn, module::{IoKind, ModuleInfo as FfiModuleInfo}}, utils::strings::cchar_to_string};

pub struct Module {
    /// A library handle needs to be stored in order to keep the imported functions available
    _lib: Library,
    pub module_info: ModuleInfo,
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
        let module_info: ModuleInfo = unsafe {
            let torustiq_module_get_info: Symbol<ModuleGetInfoFn> = lib.get(b"torustiq_module_get_info")?;
            torustiq_module_get_info()
        }.into();

        Ok(Module {
            _lib: lib,
            module_info,
        })
    }
}