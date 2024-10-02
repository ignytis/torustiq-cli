use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{
        atomic::{
            AtomicUsize, Ordering
        },
        Arc, Mutex
    },
    thread, time::Duration
};

use log::info;

use torustiq_common::ffi::{
    shared::torustiq_module_free_record,
    types::{
        module::{ModuleStepConfigureArgs, ModuleStepHandle, PipelineStepKind, Record},
        std_types, traits::ShallowCopy
    }
};

use crate::{
    callbacks,
    config::PipelineDefinition,
    module::Module,
    pipeline::pipeline_step::PipelineStep,
    shutdown::is_termination_requested,
    xthread::{FREE_BUF, SENDERS}
};

pub struct Pipeline {
    /// Stores the number of active reader threads. Needed to check if the app could be terminated.
    reader_threads_count: Arc<AtomicUsize>,
    pub steps: Vec<Arc<Mutex<PipelineStep>>>,
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
            .map(|(step_index, step_def)| {
                let s = PipelineStep::from_module(
                    modules.get(&step_def.handler).unwrap().clone(),
                    step_index, step_def.args.clone());
                Arc::new(Mutex::new(s))
            })
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
    pub fn configure_steps(&mut self) -> Result<(), String> {
        info!("Configuring steps...");
        let last_step_index = self.steps.len() - 1;
        for (step_index, step_mtx) in self.steps.iter_mut().enumerate() {
            let mut step = step_mtx.lock().unwrap();
            let step_handle = step.handle;
            for (k, v) in &step.args { // set arguments for step
                step.module.set_step_param(step_handle, k, v);
            }
            let kind = if 0 == step_index { PipelineStepKind::Source }
                else if last_step_index == step_index { PipelineStepKind::Destination }
                else { PipelineStepKind::Transformation };
            step.configure(ModuleStepConfigureArgs{
                kind,
                step_handle: std_types::Uint::try_from(step_handle).unwrap(),
                on_step_terminate_cb: callbacks::on_step_terminate_cb,
                on_data_received_fn: callbacks::on_rcv_cb,
            })?;
        }
        Ok(())
    }

    /// Start senders and receivers
    pub fn start_senders_receivers(&self) {
        let mut senders = SENDERS.lock().unwrap();
        for i in 0..self.steps.len() - 1 {
            let i_sender = i;
            let i_receiver = i_sender + 1;
            let step_sender = self.steps.get(i_sender).unwrap().clone();
            let step_receiver = self.steps
                .get(i_receiver).unwrap()
                .lock().unwrap();

            let i_sender_ffi = u32::try_from(i_sender).unwrap();
            let i_receiver_ffi = u32::try_from(i_receiver).unwrap();

            let process_record_ptr = step_receiver.module.process_record_ptr.clone();
            let step_shutdown_ptr = step_receiver.module.step_shutdown_ptr.clone();

            let (tx, rx) = std::sync::mpsc::channel::<Record>();
            senders.insert(i_sender_ffi, tx);

            FREE_BUF.lock().unwrap().insert(i_sender_ffi, *step_sender.lock().unwrap().module.free_record_ptr.clone());
            let reader_threads_count = self.reader_threads_count.clone();
            reader_threads_count.fetch_add(1, Ordering::SeqCst);
            // Reader thread
            thread::spawn(move || {
                loop {
                    let record = match rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(r) => r,
                        Err(_) => { // timeout
                            if step_sender.lock().unwrap().is_terminated() { // no messages because the source is shut down
                                break;
                            } else {
                                continue; // no messages, but source is online
                            }
                        }
                    };
                    // TODO: is deep copy + deallocation of original better?
                    // in this case we don't have to worry about updates by references inside process_record_ptr
                    let record_copy = record.shallow_copy();
                    process_record_ptr(record, i_receiver_ffi);
                    torustiq_module_free_record(record_copy);
                }
                // while !is_termination_requested() {

                // }

                // Processed all the data from upstream. Terminating the current step
                // step_receiver.module.shutdown(i_receiver);
                step_shutdown_ptr(i_receiver_ffi);
                reader_threads_count.fetch_sub(1, Ordering::SeqCst);
            });
        }
    }

    /// Starts the data processing routines inside each step
    pub fn start_steps(&self) -> Result<(), String> {
        info!("Starting steps...");
        for step_mtx in &self.steps {
            let step = step_mtx.lock().unwrap();
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
    /// "running" means "not all steps are terminated"
    pub fn is_running(&self) -> bool {
        let steps_total = self.steps.len();
        let steps_terminated = self.steps.iter()
            .map(|step| match step.lock().unwrap().is_terminated() { true => 1, false => 0 })
            .fold(0, |acc, e| acc + e );

        steps_terminated < steps_total
        // TODO: change this. Need to gracefully shut down steps and set a flag
        //self.reader_threads_count.load(Ordering::SeqCst) > 0
    }

    pub fn get_step_by_handle_mut(&self, handle: usize) -> Option<Arc<Mutex<PipelineStep>>> {
        for h in &self.steps {
            if h.lock().unwrap().handle == handle {
                return Some(h.clone())
            }
        }
        None
    }
}