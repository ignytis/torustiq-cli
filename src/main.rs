use std::collections::HashMap;
use std::{env, error::Error, fs, io, str};

use clap::{arg, command, Parser};
use libloading::Library;
use log::debug;
use serde::{Serialize, Deserialize};

use torustiq_common::ffi;
use torustiq_common::types::module::ModuleInfo;

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

    let pipeline_cfg: String = match fs::read_to_string(&args.pipeline_file) {
        Ok(c) => c,
        Err(e) => return err(format!("Cannot open the pipeline file: '{}'. {}", args.pipeline_file, e)),
    };
    let pipeline_cfg: PipelineDefinition = match serde_yaml::from_str(pipeline_cfg.as_str()) {
        Ok(c) => c,
        Err(e) => return err(format!("Cannot parse the pipeline: '{}'. {}", args.pipeline_file, e)),
    };
    let required_module_ids: Vec<String> = pipeline_cfg.steps
        .iter()
        .map(|step| step.handler.clone())
        .collect();

    for entry in fs::read_dir(args.module_dir)? {
        let entry = entry?;
        let path = entry.path();

        // TODO: insead of saving the handle, we can initialize a module straight away
        let mut module_mapping: HashMap<String, Library> = HashMap::new(); // key: module ref ID, value: library handle
        unsafe {
            let lib = libloading::Library::new(&path)?;
            let func: libloading::Symbol<unsafe extern fn() -> ffi::types::module::ModuleInfo> = lib.get(b"get_info")?;
            let module: ModuleInfo = func().into();
            let path_str = path.into_os_string().into_string().unwrap();
            
            debug!("Module info received: {} (ref_id: {}, path: {})", module.name, module.ref_id, path_str);
            if required_module_ids.contains(&module.ref_id) {
                debug!("Added module '{}' to loading list as it is referred in the pipeline definition", module.ref_id);
                module_mapping.insert(module.ref_id.clone(), lib);
            } else {
                debug!("Skipped module '{}' from loading as it is not referred in the pipeline definition", module.ref_id);
            }
        }
    }

    Ok(())
}

fn err(message: String) -> Result<(), Box<dyn Error>> {
    Err(Box::new(io::Error::new(io::ErrorKind::Other, message)))
}