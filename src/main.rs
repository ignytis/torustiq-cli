pub mod cli;
pub mod module;
pub mod module_loader;
pub mod pipeline;
pub mod shutdown;

use std::{
    collections::HashMap, convert::TryFrom, fs,
    process::exit,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::Sender, Mutex
    },
    thread, time::{self, Duration}
    };

use log::{debug, error, info};
use once_cell::sync::Lazy;

use shutdown::{init_signal_handler, is_termination_requested, on_terminate_cb};
use torustiq_common::{
    ffi::{shared::torustiq_module_free_record,types::{
            functions::ModuleFreeRecordFn,
            module::{ModuleStepHandle, ModuleStepInitArgs, PipelineStepKind, Record},
            std_types, traits::ShallowCopy
        }},
    logging::init_logger
};

use crate::{
    cli::CliArgs,
    module_loader::load_modules,
    pipeline::{Pipeline, PipelineDefinition}
};

/// Stores the number of pipeline threads. A pipeline thread is created per pipeline step
static PIPELINE_THREADS_COUNT: AtomicUsize = AtomicUsize::new(0);
static PIPELINE: Lazy<Mutex<Option<Pipeline>>>= Lazy::new(|| {
    Mutex::new(None)
});
static SENDERS: Lazy<Mutex<HashMap<ModuleStepHandle, Sender<Record>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});
/// A hashmap of pointers to torustiq_module_free_record functions.
/// As records must be deallocated by modules they are created in, this map allows to
/// locate a deallocation function owned by module
static FREE_BUF: Lazy<Mutex<HashMap<ModuleStepHandle, ModuleFreeRecordFn>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// Modules use this function to send a message
extern "C" fn on_rcv(record: Record, step_handle: ModuleStepHandle) {
    let sender = match SENDERS.lock().unwrap().get(&step_handle) {
        Some(s) => s.clone(),
        None => return, // no sender exists: no action
    };

    // Sends a cloned record to further processing and deallocates the original record
    sender.send(record.clone()).unwrap();
    let free_buf_fn_map = FREE_BUF.lock().unwrap();
    let free_buf_fn = free_buf_fn_map.get(&step_handle).unwrap();
    free_buf_fn(record);
}

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

    let pipeline = match Pipeline::from_definition(&pipeline_def, &modules) {
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to create a pipeline from definition: {}", e))
    };
    info!("Constructed a pipeline which contains {} steps", pipeline.steps.len());
    Ok(pipeline)
}

/// Initialization of steps: opens files or DB connections, starts listening sockets, etc
fn initialize_steps(pipeline: &Pipeline) -> Result<(), String> {
    info!("Initialization of steps...");
    let last_step_index = pipeline.steps.len() - 1;
    for (step_index, step) in pipeline.steps.iter().enumerate() {
        let step_handle = step.handle;
        for (k, v) in &step.args { // set arguments for step
            step.module.set_step_param(step_handle, k, v);
        }
        let kind = if 0 == step_index { PipelineStepKind::Source }
            else if last_step_index == step_index { PipelineStepKind::Destination }
            else { PipelineStepKind::Transformation };
        let init_args = ModuleStepInitArgs{
            kind,
            step_handle: std_types::Uint::try_from(step_handle).unwrap(),
            termination_handler: on_terminate_cb,
            on_data_received_fn: on_rcv,
        };
        // TODO:
        // 1. Configure - pass configuration without starting servers / threads / etc
        // 2. Start - running steps
        match step.module.init_step(init_args) {
            Ok(_) => {},
            Err(msg) => {
                return Err(format!("Failed to load step {}: {}", step.id, msg));
            }
        }
    }
    Ok(())
}

/// Initialize senders and receivers
fn start_senders_receivers(pipeline: &Pipeline) {
    let mut senders = SENDERS.lock().unwrap();
    for i in 0..pipeline.steps.len() - 1 {
        let i_sender = i;
        let i_receiver = i_sender + 1;
        let step_sender = pipeline.steps.get(i_sender).unwrap();
        let step_receiver = pipeline.steps.get(i_receiver).unwrap();

        let i_sender_ffi = u32::try_from(i_sender).unwrap();
        let i_receiver_ffi = u32::try_from(i_receiver).unwrap();

        let process_record_ptr = step_receiver.module.process_record_ptr.clone();

        let (tx, rx) = std::sync::mpsc::channel::<Record>();
        senders.insert(i_sender_ffi, tx);

        FREE_BUF.lock().unwrap().insert(i_sender_ffi, *step_sender.module.free_record_ptr.clone());
        PIPELINE_THREADS_COUNT.fetch_add(1, Ordering::SeqCst);
        thread::spawn(move|| {
            while !is_termination_requested() {
                let record = match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(r) => r,
                    Err(_) => continue, // timeout
                };
                // TODO: is deep copy + deallocation of original better?
                let record_copy = record.shallow_copy();
                process_record_ptr(record, i_receiver_ffi);
                torustiq_module_free_record(record_copy);
            }
            PIPELINE_THREADS_COUNT.fetch_sub(1, Ordering::SeqCst);
        });
    }
}

fn main() {
    init_logger();
    info!("Starting the application...");
    match init_signal_handler() {
        Ok(_) => {},
        Err(msg) => return crash_with_message(msg),
    };

    let args = CliArgs::do_parse();
    {
        let mut pipeline_option = PIPELINE.lock().unwrap();
        *pipeline_option = match create_pipeline(&args) {
            Ok(p) => Some(p),
            Err(msg) => return crash_with_message(format!("Failed to create a pipeline: {}", msg))
        };
        let pipeline = pipeline_option.as_mut().unwrap();
        match initialize_steps(pipeline) {
            Err(msg) => return crash_with_message(format!("Cannot initialize steps: {}", msg)),
            _ => {},
        };
        start_senders_receivers(pipeline);
    }

    while PIPELINE_THREADS_COUNT.load(Ordering::SeqCst) > 0 {
        thread::sleep(time::Duration::from_millis(100));
    }
    debug!("Exited from main loop");

    info!("Application terminated.");
}

fn crash_with_message(msg: String) {
    error!("An error occurred. {}", msg);
    exit(-1);
}