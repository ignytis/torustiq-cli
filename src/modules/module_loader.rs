use std::{
    collections::HashMap,
    fs, sync::Arc};

use log::debug;
use libloading::{Library, Symbol};

use torustiq_common::ffi::{
    types::functions::ModuleGetInfoFn,
    utils::strings::cchar_to_string,
};

use crate::{
    config::PipelineDefinition,
    modules::step_module::StepModule,
};

/// Returns a HashMap of modules referenced in the pipeline definition
pub fn load_modules(module_dir: &String, pipeline_def: &PipelineDefinition) -> Result<HashMap<String, Arc<StepModule>>, String> {
    let mut modules: HashMap<String, Arc<StepModule>> = HashMap::new();
    let required_module_ids: Vec<String> = pipeline_def.steps
        .iter()
        .map(|step| step.handler.clone())
        .collect();

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
            let torustiq_module_get_info: Symbol<ModuleGetInfoFn> = match lib.get(b"torustiq_module_get_info") {
                Ok(s) => s,
                Err(e) => return Err(format!("Failed to load function 'torustiq_module_get_info' from library '{}': {}", path_str, e)),
            };
            (torustiq_module_get_info(), lib)
        };
        let module_id = cchar_to_string(module_info.id);
        debug!("Module at path {} identified: {}", path_str, module_id);
        if !required_module_ids.contains(&module_id) {
            debug!("Skipped module '{}' because it doesn't exist in the pipeline", module_id);
            continue
        }

        let module = match StepModule::try_from(lib) {
            Ok(m) => m,
            Err(e) => return Err(format!("Failed to initialize a module from library '{}': {}", path_str, e)),
        };
        modules.insert(module_id.clone(), Arc::from(module));
        debug!("Module '{}' is loaded.", module_id);
    }

    let loaded_module_ids: Vec<String> = modules.keys().cloned().collect();
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

    Ok(modules)
}
