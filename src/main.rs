use std::collections::HashMap;
use std::{env, error::Error, fs, io, str};

use clap::{arg, command, Parser};
use log::debug;
use serde::{Serialize, Deserialize};

use torustiq_common::ffi::{
    types::functions::{GetIdFn, LoadFn},
    utils::strings::cchar_to_string,
};
use torustiq_common::types::module::Module;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct PipelineStep {
    name: String,
    handler: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct PipelineDefinition {
    steps: Vec<PipelineStep>,
}

/// Starts a data processing pipeline from provided config
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// A YAML file to read the pipeline structure from
    #[arg(short, long, default_value="pipeline.yaml")]
    pipeline_file: String,

    /// A YAML file to read the pipeline structure from
    #[arg(short, long, default_value="modules")]
    module_dir: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();

    let args = Args::parse();

    let pipeline_def: String = match fs::read_to_string(&args.pipeline_file) {
        Ok(c) => c,
        Err(e) => return err(format!("Cannot open the pipeline file: '{}'. {}", args.pipeline_file, e)),
    };
    let pipeline_def: PipelineDefinition = match serde_yaml::from_str(pipeline_def.as_str()) {
        Ok(c) => c,
        Err(e) => return err(format!("Cannot parse the pipeline: '{}'. {}", args.pipeline_file, e)),
    };

    let modules = load_modules(&args.module_dir, &pipeline_def)?;
    for (module_id, _module) in &modules {
        debug!("Module: {}", module_id)
    }

    Ok(())
}

fn load_modules(module_dir: &String, pipeline_def: &PipelineDefinition) -> Result<HashMap<String, Module>, Box<dyn Error>> {
    let mut modules: HashMap<String, Module> = HashMap::new();
    let required_module_ids: Vec<String> = pipeline_def.steps
        .iter()
        .map(|step| step.handler.clone())
        .collect();

    for entry in fs::read_dir(module_dir)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.clone().into_os_string().into_string().unwrap();

        // TODO: insead of saving the handle, we can initialize a module straight away
        // let mut module_mapping: HashMap<String, Library> = HashMap::new(); // key: module ref ID, value: library handle
        unsafe {
            let lib = libloading::Library::new(&path)?;
            let get_id_fn: libloading::Symbol<GetIdFn> = lib.get(b"get_id")?;
            let module_id = cchar_to_string(get_id_fn());
            debug!("Module at path {} identified: {}", path_str, module_id);
            if required_module_ids.contains(&module_id) {
                debug!("Loading module '{}'...", module_id);
                let load_fn: libloading::Symbol<LoadFn> = lib.get(b"load")?;
                let module: Module = load_fn().into();

                modules.insert(module_id.clone(), module);
            } else {
                debug!("Skipped module '{}' because it doesn't exist in the pipeline", module_id);
            }
        }
    }

    Ok(modules)
}

fn err(message: String) -> Result<(), Box<dyn Error>> {
    Err(Box::new(io::Error::new(io::ErrorKind::Other, message)))
}