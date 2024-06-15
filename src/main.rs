pub mod cli;
pub mod module;
pub mod pipeline;

use std::{
    collections::HashMap,
    convert::TryFrom,
    error::Error, fs, io, rc::Rc, thread, time};

use log::{debug, info};
use libloading::{Library, Symbol};

use module::Module;
use torustiq_common::{
    ffi::{
        types::{
            functions::ModuleGetInfoFn,
            module::ModuleInitStepArgs, std_types,
        },
        utils::strings::cchar_to_string,
    }, logging::init_logger
};

use crate::{
    cli::CliArgs,
    pipeline::{Pipeline, PipelineDefinition}
};

static mut TODO_TERMINATE: bool = false;

extern "C" fn on_terminate(step_handle: std_types::Uint) {
    info!("Received a termination signal from step with index {}", step_handle);
    unsafe {
        TODO_TERMINATE = true;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    info!("Starting the application...");
    let args = CliArgs::do_parse();

    let pipeline_def: String = match fs::read_to_string(&args.pipeline_file) {
        Ok(c) => c,
        Err(e) => return err(format!("Cannot open the pipeline file: '{}'. {}", args.pipeline_file, e)),
    };
    let pipeline_def: PipelineDefinition = match serde_yaml::from_str(pipeline_def.as_str()) {
        Ok(c) => c,
        Err(e) => return err(format!("Cannot parse the pipeline: '{}'. {}", args.pipeline_file, e)),
    };

    // _libraries is needed to keep libraries loaded into memory
    let modules = load_modules(&args.module_dir, &pipeline_def)?;
    info!("All modules are loaded");

    let pipeline = match Pipeline::build(&pipeline_def, &modules) {
        Ok(p) => p,
        Err(msg) => return err(msg),
    };
    info!("Constructed a pipeline which contains {} steps", pipeline.steps.len());

    // Initialization of steps: opens files or DB connections, starts listening sockets, etc
    info!("Initialization of steps...");
    for (step_index, step) in pipeline.steps.iter().enumerate() {
        step.module.init_step(ModuleInitStepArgs{
            step_handle: std_types::Uint::try_from(step_index).unwrap(),
            termination_handler: on_terminate,
        })?;
    }

    unsafe {
        while !TODO_TERMINATE {
            thread::sleep(time::Duration::from_millis(10));
        }
    }
    debug!("Exited from main loop");

    info!("Application terminated.");
    Ok(())
}

fn load_modules(module_dir: &String, pipeline_def: &PipelineDefinition) -> Result<HashMap<String, Rc<Module>>, Box<dyn Error>> {
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

        unsafe {
            let lib = Library::new(&path)?;
            let torustiq_module_get_info: Symbol<ModuleGetInfoFn> = lib.get(b"torustiq_module_get_info")?;
            let module_info = torustiq_module_get_info();
            let module_id = cchar_to_string(module_info.id);
            debug!("Module at path {} identified: {}", path_str, module_id);
            if !required_module_ids.contains(&module_id) {
                debug!("Skipped module '{}' because it doesn't exist in the pipeline", module_id);
                continue
            }

            modules.insert(module_id.clone(), Rc::from(Module::from_library(lib)?));
            debug!("Module '{}' is loaded.", module_id);
        }
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
        return err(format!("Failed to load modules: {}", missing_module_ids.join(", ")))
    }

    Ok(modules)
}

fn err<T>(message: String) -> Result<T, Box<dyn Error>> {
    Err(Box::new(io::Error::new(io::ErrorKind::Other, message)))
}