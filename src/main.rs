pub mod cli;
pub mod module;
pub mod module_loader;
pub mod pipeline;

use std::{
    collections::HashMap, convert::TryFrom, fs, sync::{mpsc::Sender, Mutex}, thread, time::{self, Duration}};

use log::{debug, info};
use once_cell::sync::Lazy;

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

static PIPELINE: Lazy<Mutex<Pipeline>> = Lazy::new(|| {
    Mutex::new(Pipeline::new())
});

static SENDERS: Lazy<Mutex<HashMap<ModuleStepHandle, Sender<Record>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

extern "C" fn on_terminate(step_handle: std_types::Uint) {
    info!("Received a termination signal from step with index {}", step_handle);
    unsafe {
        TODO_TERMINATE = true;
    }
}

extern "C" fn on_rcv(record: Record, step_handle: ModuleStepHandle) {
    let sender = match SENDERS.lock().unwrap().get(&step_handle) {
        Some(s) => s.clone(),
        None => return,
    };

    sender.send(record).unwrap();
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

        // Initialize senders and receivers
        for i in 0..pipeline.steps.len() - 1 {
            let i_sender = i;
            let i_receiver = i_sender + 1;
            let step_receiver = pipeline.steps.get(i_receiver).unwrap();

            let i_sender_ffi = u32::try_from(i_sender).unwrap();
            let i_receiver_ffi = u32::try_from(i_receiver).unwrap();

            let process_record_ptr = step_receiver.module.process_record_ptr.clone();

            let (tx, rx) = std::sync::mpsc::channel::<Record>();
            SENDERS.lock().unwrap().insert(i_sender_ffi, tx);
            thread::spawn(move|| {
                let todo_terminate = unsafe{ TODO_TERMINATE };
                while !todo_terminate {
                    let record = match rx.recv_timeout(Duration::from_secs(1)) {
                        Ok(r) => r,
                        Err(_) => continue, // timeout
                    };
                    process_record_ptr(record, i_receiver_ffi);
                }
            });
        }
    }
    info!("Before main loop");

    unsafe {
        while !TODO_TERMINATE {
            thread::sleep(time::Duration::from_millis(10));
        }
    }
    debug!("Exited from main loop");

    info!("Application terminated.");
}