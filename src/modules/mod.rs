pub mod module_loader;
pub mod step_module;

use libloading::Library;
#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;


use torustiq_common::ffi::{types::{
    functions as fn_defs,
    module::ModuleInfo as FfiModuleInfo,
    module::ModuleKind as FfiModuleKind,
}, utils::strings::cchar_to_string};


/// Defines the kind of module.
pub enum ModuleKind {
    /// A pipeline step module. Extracts, transforms, loads the data.
    Step,
    /// An event listener module. Reacts to application events.
    EventListener,
}

pub struct ModuleInfo {
    /// API version is used to verify the compatibility between host app and module
    pub api_version: u32,
    /// Module ID must be unique for all modules.
    /// This ID is used to match configuration with loaded modules.
    /// It's not recommended to use upper-case characters, special symbols and spaces in ID.
    pub id: String,
    pub kind: ModuleKind,
    /// A human-readable name. Unlike ID, it might contain multiple words
    pub name: String,
}

impl From<FfiModuleKind> for ModuleKind {
    fn from(value: FfiModuleKind) -> ModuleKind {
        match value {
            FfiModuleKind::EventListener => ModuleKind::EventListener,
            FfiModuleKind::Step => ModuleKind::Step,
        }
    }
}

impl From<FfiModuleInfo> for ModuleInfo {
    fn from(value: FfiModuleInfo) -> ModuleInfo {
        ModuleInfo {
            api_version: value.api_version,
            id: cchar_to_string(value.id),
            kind: value.kind.into(),
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