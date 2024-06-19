pub mod cli;
pub mod module;
pub mod module_loader;
pub mod pipeline;

use std::{
    convert::TryFrom,
    fs, sync::Mutex, thread, time};

use log::{debug, info, warn};

use torustiq_common::{
    ffi::types::{
            module::{ModuleInitStepArgs, ModuleStepHandle, Record},
            std_types,
        },
    logging::init_logger
};

use crate::{
    cli::CliArgs,
    module_loader::load_modules,
    pipeline::{Pipeline, PipelineDefinition}
};

static mut TODO_TERMINATE: bool = false;

use once_cell::sync::Lazy;

static PIPELINE: Lazy<Mutex<Pipeline>> = Lazy::new(|| {
    Mutex::new(Pipeline::new())
});

extern "C" fn on_terminate(step_handle: std_types::Uint) {
    info!("Received a termination signal from step with index {}", step_handle);
    unsafe {
        TODO_TERMINATE = true;
    }
}

extern "C" fn on_rcv(record: Record, step_handle: ModuleStepHandle) {
    // Send data to the next module
    let pipeline = PIPELINE.lock().unwrap();
    let next_step_index: usize = ModuleStepHandle::try_into(step_handle + 1).unwrap();
    let step = match pipeline.steps.get(next_step_index) {
        Some(s) => s,
        None => {
            warn!("Cannot submit the record to step {}: out of range ({})", next_step_index, pipeline.steps.len());
            return
        }
    };
    step.module.process_record(record);
}

fn main() {
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

    let modules = load_modules(&args.module_dir, &pipeline_def);
    info!("All modules are loaded");

    {
        let mut pipeline = PIPELINE.lock().unwrap();
        pipeline.build_steps(&pipeline_def, &modules);
        info!("Constructed a pipeline which contains {} steps", pipeline.steps.len());

        // Initialization of steps: opens files or DB connections, starts listening sockets, etc
        info!("Initialization of steps...");
        for (step_index, step) in pipeline.steps.iter().enumerate() {
            step.module.init_step(ModuleInitStepArgs{
                step_handle: std_types::Uint::try_from(step_index).unwrap(),
                termination_handler: on_terminate,
                on_data_received_fn: on_rcv,
            });
        }
    }

    unsafe {
        while !TODO_TERMINATE {
            thread::sleep(time::Duration::from_millis(10));
        }
    }
    debug!("Exited from main loop");

    info!("Application terminated.");
}