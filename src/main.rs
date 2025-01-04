pub mod callbacks;
pub mod cli;
pub mod config;
pub mod modules;
pub mod pipeline;
pub mod shutdown;
pub mod xthread;

use std::{
    fs, process::exit, sync::{Arc, Mutex}, thread, time
};

use libloading::Library;
use log::{debug, error, info};

use shutdown::init_signal_handler;
use torustiq_common::logging::init_logger;
use xthread::PIPELINE;

use crate::{
    cli::CliArgs,
    config::PipelineDefinition,
    modules::module_loader::load_libraries,
    pipeline::pipeline::Pipeline
};

/// Creates a pipeline from pipeline definition file
fn create_pipeline(args: &CliArgs) -> Result<(Pipeline, Vec<Library>), String> {
    debug!("Creating a pipeline from definition file: {}", &args.pipeline_file);
    let pipeline_def: String = match fs::read_to_string(&args.pipeline_file) {
        Ok(c) => c,
        Err(e) => return Err(format!("Cannot open the pipeline file: '{}'. {}", args.pipeline_file, e)),
    };
    let pipeline_def: PipelineDefinition = match serde_yaml::from_str(pipeline_def.as_str()) {
        Ok(c) => c,
        Err(e) => return Err(format!("Cannot parse the pipeline: '{}'. {}", args.pipeline_file, e)),
    };

    let module_ids_required = pipeline_def.get_module_ids_in_use();
    let loaded_libs: modules::module_loader::LoadedLibraries = load_libraries(&args.module_dir, module_ids_required)?;
    info!("All modules are loaded.");
    loaded_libs.init();

    let pipeline = match Pipeline::try_from((&pipeline_def, &loaded_libs)) {
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to create a pipeline from definition: {}", e))
    };
    info!("Constructed a pipeline which contains {} steps", pipeline.steps.len());
    let loaded_libs = loaded_libs.libs;
    Ok((pipeline, loaded_libs))
}

fn main() {
    init_logger();
    info!("Starting the application...");
    if let Err(msg) = init_signal_handler() {
        return crash_with_message(msg)
    };

    let args = CliArgs::do_parse();
    let (pipeline, _loaded_libs) = match create_pipeline(&args) {
        Ok(p) => p,
        Err(msg) => return crash_with_message(format!("Failed to create a pipeline: {}", msg))
    };
    if pipeline.description.is_some() {
        debug!("Description of pipeline: {}", pipeline.description.clone().unwrap());
    }
    
    let pipeline_arc = Arc::new(Mutex::new(pipeline));

    if let Err(_) = PIPELINE.set(pipeline_arc.clone()) {
        crash_with_message(format!("Failed to register the pipeline in static context"))
    }

    {
        let mut pipeline = pipeline_arc.lock().unwrap();

        if let Err(msg) = pipeline.configure_steps() {
            return crash_with_message(format!("Cannot configure steps: {}", msg));
        };

        if let Err(msg) = pipeline.configure_listeners() {
            return crash_with_message(format!("Cannot configure listeners: {}", msg));
        };

        if let Err(msg) = pipeline.start_senders_receivers() {
            return crash_with_message(format!("Cannot start the sender and receiver channels: {}", msg));
        };

        if let Err(msg) = pipeline.start_steps() {
            return crash_with_message(format!("Cannot start steps: {}", msg))
        };
    }

    while pipeline_arc.lock().unwrap().is_running() {
        thread::sleep(time::Duration::from_millis(100));
    }
    debug!("Exited from main loop");

    info!("Application terminated.");
}

fn crash_with_message(msg: String) {
    error!("An error occurred. {}", msg);
    exit(-1);
}