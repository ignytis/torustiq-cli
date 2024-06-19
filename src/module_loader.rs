use std::{
    collections::HashMap,
    error::Error, fs, rc::Rc};

use log::debug;
use libloading::{Library, Symbol};

use torustiq_common::ffi::{
    types::functions::ModuleGetInfoFn,
    utils::strings::cchar_to_string,
};

use crate::{
    module::Module,
    pipeline::PipelineDefinition
};

pub fn load_modules(module_dir: &String, pipeline_def: &PipelineDefinition) -> Result<HashMap<String, Rc<Module>>, Box<dyn Error>> {
    let mut modules: HashMap<String, Rc<Module>> = HashMap::new();
    // let mut libraries: Vec<Library> = Vec::new();
    let required_module_ids: Vec<String> = pipeline_def.steps
        .iter()
        .map(|step| step.handler.clone())
        .collect();

    for entry in fs::read_dir(module_dir)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.clone().into_os_string().into_string().unwrap();

        let (module_info, lib) = unsafe {
            let lib = Library::new(&path)?;
            let torustiq_module_get_info: Symbol<ModuleGetInfoFn> = lib.get(b"torustiq_module_get_info")?;
            (torustiq_module_get_info(), lib)
        };
        let module_id = cchar_to_string(module_info.id);
        debug!("Module at path {} identified: {}", path_str, module_id);
        if !required_module_ids.contains(&module_id) {
            debug!("Skipped module '{}' because it doesn't exist in the pipeline", module_id);
            continue
        }

        modules.insert(module_id.clone(), Rc::from(Module::from_library(lib)?));
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
        panic!("Failed to load modules: {}", missing_module_ids.join(", "))
    }

    Ok(modules)
}
