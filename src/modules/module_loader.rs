use std::{collections::HashMap, fs, sync::Arc};
use std::error::Error;

use libloading::{Library, Symbol};
#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;

use log::{debug, info, warn};

use torustiq_common::{
    ffi::{
        types::functions as fn_defs,
        utils::strings::cchar_to_string,
    },
    CURRENT_API_VERSION
};

use crate::modules::{
    BaseModule, ModuleInfo, ModuleKind,
    pipeline::PipelineModule,
    event_listener::EventListenerModule
};

#[derive(Default)]
pub struct LoadedLibraries {
    pub event_listeners: HashMap<String, Arc<EventListenerModule>>,
    pub pipeline: HashMap<String, Arc<PipelineModule>>,
}

impl LoadedLibraries {
    pub fn init(&self) {
        info!("Initialization of modules...");
        for module in self.event_listeners.values() {
            debug!("Initializing event listener module '{}' (name: '{}')...", module.get_info().id, module.get_info().name);
            module.init();
        }
        for module in self.pipeline.values() {
            debug!("Initializing step module '{}' (name: '{}')...", module.get_info().id, module.get_info().name);
            module.init();
        }
    }
}

/// Returns a HashMap of modules referenced in the pipeline definition
pub fn load_modules(module_dir: &String, required_module_ids: Vec<String>) -> Result<LoadedLibraries, String> {
    let mut loaded_libs = LoadedLibraries::default();
    let mut loaded_module_ids: Vec<String> = Vec::new();

    let dir = match fs::read_dir(module_dir) {
        Ok(d) => d,
        Err(e) => return Err(format!("Cannot open directory '{}': {}", module_dir, e))
    };
    for entry in dir {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => return Err(format!("Failed to load an entry: {}", e)),
        };
        let path = entry.path();
        let path_str = match path.clone().into_os_string().into_string() {
            Ok(p) => p,
            Err(e) => return Err(format!("Failed to convert path into string: {:?}", e)),
        };

        let (module_info, lib) = unsafe {
            let lib = match Library::new(&path) {
                Ok(l) => l,
                Err(e) => return Err(format!("Failed to load a library at path '{}': {}", path_str, e)),
            };
            let torustiq_module_get_info: Symbol<fn_defs::ModuleGetInfoFn> = match lib.get(b"torustiq_module_get_info") {
                Ok(s) => s,
                Err(e) => return Err(format!("Failed to load function 'torustiq_module_get_info' from library '{}': {}", path_str, e)),
            };
            (torustiq_module_get_info(), lib)
        };
        if module_info.api_version != CURRENT_API_VERSION {
            warn!("Library '{}' is skipped because it has API version {} which is incompatible with current application's API version {}",
                path_str, module_info.api_version, CURRENT_API_VERSION);
            continue
        }
        let module_id = cchar_to_string(module_info.id);
        debug!("Module at path {} identified: {}", path_str, module_id);
        if !required_module_ids.contains(&module_id) {
            debug!("Skipped module '{}' because it doesn't exist in the pipeline", module_id);
            continue
        }

        match load_module(lib) {
            Ok(loaded_lib) => {
                match loaded_lib {
                    LoadedLibrary::Pipeline(p) => {
                        loaded_libs.pipeline.insert(module_id.clone(), Arc::from(p));
                    },
                    LoadedLibrary::EventListener(l) => {
                        loaded_libs.event_listeners.insert(module_id.clone(), Arc::from(l));
                    },
                };
                loaded_module_ids.push(module_id.clone());
                debug!("Module '{}' is loaded.", module_id);
            },
            Err(e) => return Err(format!("Failed to initialize a module from library '{}': {}", path_str, e)),
        };
    }

    let missing_module_ids: Vec<String> = required_module_ids
        .into_iter()
        .filter(|item| !loaded_module_ids.contains(item))
        .collect();
    if missing_module_ids.len() > 0 {
        for m in &missing_module_ids {
            log::error!("An unknown module is detected in pipeline: {}", m);
        }
        return Err(format!("Failed to load modules: {}", missing_module_ids.join(", ")));
    }

    Ok(loaded_libs)
}

/// Loads a module from library
fn load_module(lib: Library) -> Result<LoadedLibrary, Box<dyn Error>> {
    let loader = RawPointerLoader::new(&lib);
    let module_info: ModuleInfo = {
        let torustiq_module_get_info: RawSymbol<fn_defs::ModuleGetInfoFn> = loader.load(b"torustiq_module_get_info")?;
        torustiq_module_get_info()
    }.into();

    let module = match module_info.kind {
        ModuleKind::Pipeline => LoadedLibrary::Pipeline(PipelineModule {
            step_configure_ptr: loader.load(b"torustiq_module_step_configure")?,
            step_process_record_ptr: loader.load(b"torustiq_module_step_process_record")?,
            free_record_ptr: loader.load(b"torustiq_module_free_record")?,

            base: create_base_module(lib, module_info)?,
        }),
        ModuleKind::EventListener => LoadedLibrary::EventListener(EventListenerModule {
            step_configure_ptr: loader.load(b"torustiq_module_step_configure")?,
            step_set_param_ptr: loader.load(b"torustiq_step_set_param")?,
            step_shutdown_ptr: loader.load(b"torustiq_module_step_shutdown")?,
            step_start_ptr: loader.load(b"torustiq_module_step_start")?,
            free_char_ptr: loader.load(b"torustiq_module_free_char_ptr")?,

            base: create_base_module(lib, module_info)?,
        })
    };

    Ok(module)
}

/// Loads raw pointers to functions from library
struct RawPointerLoader<'a> {
    lib: &'a Library
}

impl<'a> RawPointerLoader<'a> {
    fn new(lib: &'a Library) -> Self {
        RawPointerLoader { lib }
    }

    /// Loads a function from library
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

pub enum LoadedLibrary {
    EventListener(EventListenerModule),
    Pipeline(PipelineModule),
}

/// Consumes library + module info and creates an instace of base module
fn create_base_module(lib: Library, module_info: ModuleInfo) -> Result<BaseModule, Box<dyn Error>> {
    let loader = RawPointerLoader::new(&lib);
    let m = BaseModule {
        init_ptr: loader.load(b"torustiq_module_init")?,
        step_set_param_ptr: loader.load(b"torustiq_step_set_param")?,
        step_shutdown_ptr: loader.load(b"torustiq_module_step_shutdown")?,
        step_start_ptr: loader.load(b"torustiq_module_step_start")?,
        free_char_ptr: loader.load(b"torustiq_module_free_char_ptr")?,

        module_info,
        _lib: lib,
    };
    Ok(m)
}