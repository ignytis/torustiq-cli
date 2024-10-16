pub mod callbacks;
pub mod cli;
pub mod config;
pub mod module;
pub mod module_loader;
pub mod pipeline;
pub mod shutdown;
pub mod xthread;

use std::{
    fs, process::exit, sync::{Arc, Mutex}, thread, time
};

use log::{debug, error, info};

use shutdown::init_signal_handler;
use torustiq_common::logging::init_logger;
use xthread::PIPELINE;

use crate::{
    cli::CliArgs,
    config::PipelineDefinition,
    module_loader::load_modules,
    pipeline::pipeline::Pipeline
};

/// Creates a pipeline from pipeline definition file
fn create_pipeline(args: &CliArgs) -> Result<Pipeline, String> {
    let pipeline_def: String = match fs::read_to_string(&args.pipeline_file) {
        Ok(c) => c,
        Err(e) => return Err(format!("Cannot open the pipeline file: '{}'. {}", args.pipeline_file, e)),
    };
    let pipeline_def: PipelineDefinition = match serde_yaml::from_str(pipeline_def.as_str()) {
        Ok(c) => c,
        Err(e) => return Err(format!("Cannot parse the pipeline: '{}'. {}", args.pipeline_file, e)),
    };

    let modules = load_modules(&args.module_dir, &pipeline_def)?;
    info!("All modules are loaded. Initialization of modules...");
    for module in modules.values() {
        debug!("Initializing module '{}'...", module.get_id());
        module.init();
    }

    let pipeline = match Pipeline::try_from((&pipeline_def, &modules)) {
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to create a pipeline from definition: {}", e))
    };
    info!("Constructed a pipeline which contains {} steps", pipeline.steps.len());
    Ok(pipeline)
}

fn main() {
    init_logger();
    info!("Starting the application...");
    match init_signal_handler() {
        Ok(_) => {},
        Err(msg) => return crash_with_message(msg),
    };

    let args = CliArgs::do_parse();
    let pipeline = match create_pipeline(&args) {
        Ok(p) => p,
        Err(msg) => return crash_with_message(format!("Failed to create a pipeline: {}", msg))
    };
    if pipeline.description.is_some() {
        debug!("Description of pipeline: {}", pipeline.description.clone().unwrap());
    }
    
    let pipeline_arc = Arc::new(Mutex::new(pipeline));

    unsafe {
        match PIPELINE.set(pipeline_arc.clone()) {
            Ok(_) => {},
            Err(_) => crash_with_message(format!("Failed to register the pipeline in static context")),
        };
    }

    {
        let mut pipeline = pipeline_arc.lock().unwrap();
        match pipeline.configure_steps() {
            Err(msg) => return crash_with_message(format!("Cannot configure steps: {}", msg)),
            _ => {},
        };

        match pipeline.start_senders_receivers() {
            Err(msg) => return crash_with_message(format!("Cannot start the sender and receiver channels: {}", msg)),
            _ => {},
        };

        match pipeline.start_steps() {
            Err(msg) => return crash_with_message(format!("Cannot start steps: {}", msg)),
            _ => {},
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