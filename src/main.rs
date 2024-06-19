pub mod cli;
pub mod module;
pub mod module_loader;
pub mod pipeline;

use std::{
    convert::TryFrom,
    error::Error, fs, thread, time};

use log::{debug, info};

use torustiq_common::{
    ffi::{
        types::{
            module::{ModuleInitStepArgs, Record}, std_types,
        },
        utils::strings::cchar_to_string,
    }, logging::init_logger
};

use crate::{
    cli::CliArgs,
    module_loader::load_modules,
    pipeline::{Pipeline, PipelineDefinition}
};

static mut TODO_TERMINATE: bool = false;

extern "C" fn on_terminate(step_handle: std_types::Uint) {
    info!("Received a termination signal from step with index {}", step_handle);
    unsafe {
        TODO_TERMINATE = true;
    }
}

extern "C" fn on_rcv(record: Record) {
    let p = cchar_to_string(record.content.get_bytes_as_const_ptr());
    info!("Got payload: {}", p);
}

fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    info!("Starting the application...");
    let args = CliArgs::do_parse();

    let pipeline_def: String = match fs::read_to_string(&args.pipeline_file) {
        Ok(c) => c,
        Err(e) => panic!("Cannot open the pipeline file: '{}'. {}", args.pipeline_file, e),
    };
    let pipeline_def: PipelineDefinition = match serde_yaml::from_str(pipeline_def.as_str()) {
        Ok(c) => c,
        Err(e) => panic!("Cannot parse the pipeline: '{}'. {}", args.pipeline_file, e),
    };

    let modules = load_modules(&args.module_dir, &pipeline_def)?;
    info!("All modules are loaded");

    let pipeline =Pipeline::build(&pipeline_def, &modules);
    info!("Constructed a pipeline which contains {} steps", pipeline.steps.len());

    // Initialization of steps: opens files or DB connections, starts listening sockets, etc
    info!("Initialization of steps...");
    for (step_index, step) in pipeline.steps.iter().enumerate() {
        step.module.init_step(ModuleInitStepArgs{
            step_handle: std_types::Uint::try_from(step_index).unwrap(),
            termination_handler: on_terminate,
            on_data_received_fn: on_rcv,
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