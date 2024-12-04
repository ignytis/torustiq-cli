pub mod event_listener;
pub mod module_loader;
pub mod pipeline;

use libloading::Library;
#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;


use torustiq_common::ffi::{
    types::{
        functions as fn_defs,
        module::{ModuleInfo as FfiModuleInfo, ModuleKind as FfiModuleKind, StepStartFnResult},
    },
    utils::strings::{cchar_const_deallocate, cchar_to_string, string_to_cchar}
};


/// Defines the kind of module.
pub enum ModuleKind {
    /// An event listener module. Reacts to application events.
    EventListener,
    /// A pipeline. Extracts, transforms, loads the data.
    Pipeline,
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
            FfiModuleKind::Pipeline => ModuleKind::Pipeline,
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
    pub step_set_param_ptr: RawSymbol<fn_defs::StepSetParamFn>,
    pub step_shutdown_ptr: RawSymbol<fn_defs::ModuleStepShutdownFn>,
    pub step_start_ptr: RawSymbol<fn_defs::StepStartFn>,
    pub free_char_ptr: RawSymbol<fn_defs::ModuleFreeCharPtrFn>,

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

    pub fn shutdown(&self, step_handle: usize) {
        (self.step_shutdown_ptr)(usize::try_into(step_handle).unwrap());
    }

    pub fn set_step_param<S: Into<String>>(&self, handle: usize, k: S, v: S) {
        let k = string_to_cchar(k);
        let v = string_to_cchar(v);
        (self.step_set_param_ptr)(usize::try_into(handle).unwrap(), k, v);

        cchar_const_deallocate(k);
        cchar_const_deallocate(v);
    }

    pub fn start_step(&self, step_handle: usize) -> Result<(), String> {
        match (self.step_start_ptr)(usize::try_into(step_handle).unwrap()) {
            StepStartFnResult::Ok => Ok(()),
            StepStartFnResult::ErrorMisc(e) => {
                let err_string = cchar_to_string(e.clone());
                (self.free_char_ptr)(e);
                Err(err_string)
            },
        }
    }

    pub fn free_c_char(&self, c: *const i8) {
        (self.free_char_ptr)(c);
    }
}