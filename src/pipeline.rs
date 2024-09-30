use std::{
    collections::HashMap, sync::Arc, str,
    convert::TryFrom,
    sync::atomic::{AtomicUsize, Ordering},
    thread, time::Duration
};


use log::info;
use serde::{Serialize, Deserialize};

use torustiq_common::ffi::{
    shared::torustiq_module_free_record,
    types::{
        module::{ModuleStepConfigureArgs, PipelineStepKind, Record},
        std_types, traits::ShallowCopy
    }
};

use crate::{
    callbacks,
    module::Module,
    shutdown::is_termination_requested,
    xthread::{FREE_BUF, SENDERS}
};

/// A step in pipeline: source, destination, transformation, etc
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PipelineStepDefinition {
    pub name: String,
    pub handler: String,
    pub args: Option<HashMap<String, String>>,
}

/// A pipeline definition. Contains multiple steps
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PipelineDefinition {
    pub steps: Vec<PipelineStepDefinition>,
}

pub struct Pipeline {
    /// Stores the number of active reader threads. Needed to check if the app could be terminated.
    reader_threads_count: Arc<AtomicUsize>,
    pub steps: Vec<PipelineStep>,
}

impl Pipeline {
    pub fn new() -> Pipeline {
        Pipeline {
            reader_threads_count: Arc::new(AtomicUsize::new(0)),
            steps: Vec::new(),
        }
    }

    /// Creates a new pipeline from definition and injects modules into steps
    pub fn from_definition(definition: &PipelineDefinition, modules: &HashMap<String, Arc<Module>>) -> Result<Self, String> {
        // Validate references to modules
        let mut pipeline = Pipeline::new();
        for step_def in &definition.steps {
            if modules.get(&step_def.handler).is_none() {
                return Err(format!("Module not found: {}", &step_def.handler));
            }
        }

        pipeline.steps = definition
            .steps
            .iter()
            .enumerate()
            .map(|(step_index, step_def)| PipelineStep::from_module(
                modules.get(&step_def.handler).unwrap().clone(),
                step_index, step_def.args.clone())  )
            .collect();
        pipeline.validate()?;

        Ok(pipeline)
    }

    /// Validates the pipeline
    fn validate(&self) -> Result<(), String> {
        // There were more validation rules initially, but almost all of them is gone after
        // the pipeline structure had been simplified
        let steps_len = self.steps.len();
        if steps_len < 2 {
            return Err(format!("Pipeline must have at least two steps. The actual number of steps: {}", steps_len))
        }
        Ok(())
    }

    /// Pass configuration to each step
    pub fn configure_steps(&self) -> Result<(), String> {
        info!("Configuring steps...");
        let last_step_index = self.steps.len() - 1;
        for (step_index, step) in self.steps.iter().enumerate() {
            let step_handle = step.handle;
            for (k, v) in &step.args { // set arguments for step
                step.module.set_step_param(step_handle, k, v);
            }
            let kind = if 0 == step_index { PipelineStepKind::Source }
                else if last_step_index == step_index { PipelineStepKind::Destination }
                else { PipelineStepKind::Transformation };
            let config_args = ModuleStepConfigureArgs{
                kind,
                step_handle: std_types::Uint::try_from(step_handle).unwrap(),
                termination_handler: callbacks::on_terminate_cb,
                on_data_received_fn: callbacks::on_rcv_cb,
            };
            match step.module.configure_step(config_args) {
                Ok(_) => {},
                Err(msg) => {
                    return Err(format!("Failed to configure step {}: {}", step.id, msg));
                }
            }
        }
        Ok(())
    }

    /// Start senders and receivers
    pub fn start_senders_receivers(&self) {
        let mut senders = SENDERS.lock().unwrap();
        for i in 0..self.steps.len() - 1 {
            let i_sender = i;
            let i_receiver = i_sender + 1;
            let step_sender: &PipelineStep = self.steps.get(i_sender).unwrap();
            let step_receiver = self.steps.get(i_receiver).unwrap();

            let i_sender_ffi = u32::try_from(i_sender).unwrap();
            let i_receiver_ffi = u32::try_from(i_receiver).unwrap();

            let process_record_ptr = step_receiver.module.process_record_ptr.clone();

            let (tx, rx) = std::sync::mpsc::channel::<Record>();
            senders.insert(i_sender_ffi, tx);

            FREE_BUF.lock().unwrap().insert(i_sender_ffi, *step_sender.module.free_record_ptr.clone());
            let reader_threads_count = self.reader_threads_count.clone();
            reader_threads_count.fetch_add(1, Ordering::SeqCst);
            thread::spawn(move|| {
                while !is_termination_requested() {
                    let record = match rx.recv_timeout(Duration::from_secs(1)) {
                        Ok(r) => r,
                        Err(_) => continue, // timeout
                    };
                    // TODO: is deep copy + deallocation of original better?
                    // in this case we don't have to worry about updates by references inside process_record_ptr
                    let record_copy = record.shallow_copy();
                    process_record_ptr(record, i_receiver_ffi);
                    torustiq_module_free_record(record_copy);
                }
                reader_threads_count.fetch_sub(1, Ordering::SeqCst);
            });
        }
    }

    /// Starts the data processing routines inside each step
    pub fn start_steps(&self) -> Result<(), String> {
        info!("Starting steps...");
        for step in &self.steps {
            let step_handle = step.handle;
            match step.module.start_step(step_handle.into()) {
                Ok(_) => {},
                Err(msg) => {
                    return Err(format!("Failed to start step {}: {}", step.id, msg));
                }
            }
        }
        Ok(())
    }

    /// Returns true if the pipeline is running
    pub fn is_running(&self) -> bool {
        // TODO: change this. Need to gracefully shut down steps and set a flag
        self.reader_threads_count.load(Ordering::SeqCst) > 0
    }
}

pub struct PipelineStep {
    pub handle: usize,
    pub id: String,
    pub module: Arc<Module>,
    pub args: HashMap<String, String>,
}

impl PipelineStep {
    /// Initializes a step from module (=dynamic library).
    /// Index is a step index in pipeline. Needed to format a unique step ID
    pub fn from_module(module: Arc<Module>, handle: usize, args: Option<HashMap<String, String>>) -> PipelineStep {
        PipelineStep {
            handle,
            id: format!("step_{}_{}", handle, module.module_info.id),
            module,
            args: args.unwrap_or(HashMap::new()),
        }
    }
}