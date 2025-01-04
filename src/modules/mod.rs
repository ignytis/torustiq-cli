pub mod listener;
pub mod module_loader;
pub mod pipeline;

#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;


use torustiq_common::ffi::{
    types::{
        functions as fn_defs,
        module::{LibInfo as FfiLibInfo, ModuleKind as FfiModuleKind, StepStartFnResult},
    },
    utils::strings::{cchar_const_deallocate, cchar_to_string, string_to_cchar}
};


/// Defines the kind of module.
#[derive(Clone)]
pub enum ModuleKind {
    /// An event listener module. Reacts to application events.
    Listener,
    /// A pipeline. Extracts, transforms, loads the data.
    Pipeline,
}

#[derive(Clone)]
pub struct LibInfo {
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
            FfiModuleKind::Listener => ModuleKind::Listener,
            FfiModuleKind::Pipeline => ModuleKind::Pipeline,
        }
    }
}

impl From<FfiLibInfo> for LibInfo {
    fn from(value: FfiLibInfo) -> LibInfo {
        LibInfo {
            api_version: value.api_version,
            id: cchar_to_string(value.id),
            kind: value.kind.into(),
            name: cchar_to_string(value.name),
        }
    }
}

/// Base module contains common module attributes
#[derive(Clone)]
pub struct BaseModule {
    pub set_param_ptr: RawSymbol<fn_defs::StepSetParamFn>,
    pub shutdown_ptr: RawSymbol<fn_defs::ModuleStepShutdownFn>,
    pub start_ptr: RawSymbol<fn_defs::StepStartFn>,
    pub free_char_ptr: RawSymbol<fn_defs::ModuleFreeCharPtrFn>,

    module_info: LibInfo,
}

impl BaseModule {
    pub fn get_info(&self) -> &LibInfo {
        &self.module_info
    }

    pub fn shutdown(&self, module_handle: usize) {
        (self.shutdown_ptr)(usize::try_into(module_handle).unwrap());
    }

    pub fn set_param<S: Into<String>>(&self, handle: usize, k: S, v: S) {
        let k = string_to_cchar(k);
        let v = string_to_cchar(v);
        (self.set_param_ptr)(usize::try_into(handle).unwrap(), k, v);

        cchar_const_deallocate(k);
        cchar_const_deallocate(v);
    }

    pub fn start(&self, module_handle: usize) -> Result<(), String> {
        match (self.start_ptr)(usize::try_into(module_handle).unwrap()) {
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