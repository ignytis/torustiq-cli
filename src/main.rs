pub mod cli;
pub mod pipeline;

use std::{collections::HashMap, env, error::Error, fs, io, thread, time};

use log::{debug, info};
use libloading::{Library, Symbol};

use torustiq_common::{
    ffi::{
        types::functions::{GetIdFn, LoadFn},
        utils::strings::cchar_to_string,
    },
    types::module::Module,
};

use crate::{
    cli::CliArgs,
    pipeline::{Pipeline, PipelineDefinition}
};

static mut TODO_TERMINATE: bool = false;

extern "C" fn on_terminate() {
    unsafe {
        TODO_TERMINATE = true;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();
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
    let (modules, _libraries) = load_modules(&args.module_dir, &pipeline_def)?;
    info!("All modules are loaded");

    let pipeline = match Pipeline::build(&pipeline_def, &modules) {
        Ok(p) => p,
        Err(msg) => return err(msg),
    };
    info!("Constructed a pipeline which contains {} steps", pipeline.steps.len());

    info!("Initialization of steps...");
    for step in pipeline.steps {
        // (step.init)(on_terminate);
        (step.init)(on_terminate);
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

fn load_modules(module_dir: &String, pipeline_def: &PipelineDefinition) -> Result<(HashMap<String, Module>, Vec<Library>), Box<dyn Error>> {
    let mut modules: HashMap<String, Module> = HashMap::new();
    let mut libraries: Vec<Library> = Vec::new();
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
            let get_id_fn: Symbol<GetIdFn> = lib.get(b"get_id")?;
            let module_id = cchar_to_string(get_id_fn());
            debug!("Module at path {} identified: {}", path_str, module_id);
            if !required_module_ids.contains(&module_id) {
                debug!("Skipped module '{}' because it doesn't exist in the pipeline", module_id);
                continue
            }
            debug!("Loading module '{}'...", module_id);
            let load_fn: Symbol<LoadFn> = lib.get(b"load")?;
            let module: Module = load_fn().into();
            modules.insert(module_id.clone(), module);
            libraries.push(lib);
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

    Ok((modules, libraries))
}

fn err<T>(message: String) -> Result<T, Box<dyn Error>> {
    Err(Box::new(io::Error::new(io::ErrorKind::Other, message)))
}