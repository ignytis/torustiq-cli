pub mod module_loader;
pub mod step_module;

use libloading::Library;
#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;


use torustiq_common::ffi::{types::{
    functions as fn_defs,
    module::ModuleInfo as FfiModuleInfo
}, utils::strings::cchar_to_string};

pub struct ModuleInfo {
    pub id: String,
    pub name: String,
}

impl From<FfiModuleInfo> for ModuleInfo {
    fn from(value: FfiModuleInfo) -> ModuleInfo {
        ModuleInfo {
            id: cchar_to_string(value.id),
            name: cchar_to_string(value.name),
        }
    }
}

/// Base module contains common module attributes
pub struct BaseModule {
    /// A pointer to torustiq_module_init function
    init_ptr: RawSymbol<fn_defs::ModuleInitFn>,

    /// A library handle needs to be stored in order to keep the imported functions available
    _lib: Library,
    module_info: ModuleInfo,
}

impl BaseModule {
    pub fn get_info(&self) -> &ModuleInfo {
        &self.module_info
    }

    /// General initialization of module
    pub fn init(&self) {
        (self.init_ptr)()
    }
}