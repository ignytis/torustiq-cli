use std::{env, error::Error, ffi::CStr, fs, io, str};

use clap::{arg, command, Parser};
use log::debug;
use serde::{Serialize, Deserialize};

use torustiq_common::ffi::module::Module;

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
    let _pipeline_cfg: PipelineDefinition = match serde_yaml::from_str(pipeline_cfg.as_str()) {
        Ok(c) => c,
        Err(e) => return err(format!("Cannot parse the pipeline: '{}'. {}", args.pipeline_file, e)),
    };

    for entry in fs::read_dir(args.module_dir)? {
        let entry = entry?;
        let path = entry.path();

        unsafe {
            let lib = libloading::Library::new(&path)?;
            let func: libloading::Symbol<unsafe extern fn() -> Module> = lib.get(b"load")?;
            let module = func();

            let module_name: &CStr = CStr::from_ptr(module.name);
            let module_name: &str = module_name.to_str().unwrap();
            let module_ref_id: &CStr = CStr::from_ptr(module.ref_id);
            let module_ref_id: &str = module_ref_id.to_str().unwrap();

            debug!("Module loaded: {} (ref_id: {}, path: {})", module_name, module_ref_id, path.into_os_string().into_string().unwrap());
        }
    }

    Ok(())
}

fn err(message: String) -> Result<(), Box<dyn Error>> {
    Err(Box::new(io::Error::new(io::ErrorKind::Other, message)))
}