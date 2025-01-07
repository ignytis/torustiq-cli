use std::{
    collections::HashMap, sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex
    }, thread, time::Duration
};

use log::{debug, error, info};

use torustiq_common::ffi::{
    types::{
        module::{
            ModuleListenerConfigureArgs, ModulePipelineConfigureArgs, ModulePipelineProcessRecordFnResult, PipelineModuleKind, Record,
        },
        std_types,
    }, utils::strings::cchar_to_string
};

use crate::{
    config::PipelineDefinition,
    modules::module_loader::LoadedLibraries,
    pipeline::{
        listener::Listener,
        pipeline_step::PipelineStep
    },
    xthread::{SystemMessage, FREE_BUF, PIPELINE, SENDERS, SYSTEM_MESSAGES}
};

/// Starts a system command thread.
/// System command threads change the state of pipeline. For instance, a command thread can terminate the pipeline.
fn start_system_command_thread(m_rx: Receiver<SystemMessage>) {
    // A system command thread
    thread::spawn(|| {
        let m_rx = m_rx;
        loop {
            let msg = match m_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(r) => r,
                Err(_) => continue, // timeout
            };

            match msg {
                SystemMessage::TerminateStep(module_handle) => {
                    let pipeline = match PIPELINE.get() {
                        Some(p) => p.lock().unwrap(),
                        None => {
                            error!("Cannot process the termination callback for step {}: \
                                pipeline is not registered in static context", module_handle);
                            return;
                        }
                    };
                    let pipeline_step_arc = match pipeline.get_step_by_handle_mut(usize::try_from(module_handle).unwrap()) {
                        Some(s) => s,
                        None => {
                            error!("Cannot find a pipeline step with handle '{}' in static context", module_handle);
                            return;
                        }
                    };
                    let mut pipeline_step = pipeline_step_arc.lock().unwrap();
                    pipeline_step.component.set_state_terminated();
                }
            }
        }
    });
}

/// Starts a reader thread.
/// Reader threads listen input from the previous (sender) steps and forward records to further (receiver) steps
fn start_reader_thread(step_sender_arc: Arc<Mutex<PipelineStep>>, step_receiver_arc: Arc<Mutex<PipelineStep>>, rx: Receiver<Record>, listeners: Vec<Listener>) {
    let step_rcv = step_receiver_arc.lock().unwrap().clone();
    thread::spawn(move || {
        let i_receiver_ffi = u32::try_from(step_rcv.get_handle()).unwrap();
        loop {
            let mut record = match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(r) => r,
                Err(_) => { // timeout
                    if step_sender_arc.lock().unwrap().component.is_terminated() { // no messages because the source is shut down
                        break;
                    } else {
                        continue; // no messages, but source is online
                    }
                }
            };
            for l in &listeners {
                l.ffi_on_record_received(i_receiver_ffi, &record);
            }

            // NO deep copy here for performance purposes.
            // In some occasions there is no need to have an original record deep-copied:
            // - it's used only partially (e.g. metadata only)
            // - it's processed instantly and therefore not stored inside module.
            // let record_copy = record.shallow_copy();
            let (success, is_consumed) = match step_rcv.ffi_process_record(record, i_receiver_ffi) {
                ModulePipelineProcessRecordFnResult::Ok(is_consumed) => (true, is_consumed),
                ModulePipelineProcessRecordFnResult::ErrWrongModuleHandle(handle, is_consumed) => {
                    error!("Wrong module handle '{}'", handle);
                    (false, is_consumed)
                },
                ModulePipelineProcessRecordFnResult::ErrMisc(cerr, is_consumed) => {
                    let err: String = cchar_to_string(cerr);
                    error!("Failed to process record in step '{}': {}", step_rcv.get_id(), err);
                    step_rcv.ffi_free_char(cerr);
                    (false, is_consumed)
                }
            };
            if success {
                for l in &listeners {
                    l.ffi_on_record_sent(i_receiver_ffi, &record);
                }
            } else {
                for l in &listeners {
                    l.ffi_on_record_error(i_receiver_ffi, &record);
                }
            }
            if !is_consumed {
                record.free_contents();
            }
        }

        // Processed all the data from upstream. Terminating the current step
        step_rcv.ffi_shutdown(i_receiver_ffi);
    });
}

#[derive(Default)]
pub struct Pipeline {
    pub description: Option<String>,
    pub listeners: Vec<Arc<Mutex<Listener>>>,
    pub steps: Vec<Arc<Mutex<PipelineStep>>>,
}

impl Pipeline {
    pub fn new() -> Pipeline {
        Pipeline::default()
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
            let module_handle = step.component.handle;
            for (k, v) in &step.component.args { // set arguments for step
                step.module.set_param(module_handle, k, v);
            }
            let kind = if 0 == step_index { PipelineModuleKind::Source }
                else if last_step_index == step_index { PipelineModuleKind::Destination }
                else { PipelineModuleKind::Transformation };
            step.configure(ModulePipelineConfigureArgs{
                kind,
                module_handle: std_types::Uint::try_from(module_handle).unwrap(),
            })?;
        }
        Ok(())
    }

    pub fn configure_listeners(&mut self) -> Result<(), String> {
        info!("Configuring steps...");

        // Format the pipeline info in order to pass it to listeners
        let mut pipeline_data: HashMap<String, String> = HashMap::new();
        self.listeners.iter().for_each(|l| {
            let l = l.lock().unwrap();
            let handle = l.get_handle().to_string();
            pipeline_data.insert(format!("listeners.{}.handle", &handle), handle.clone());
            pipeline_data.insert(format!("listeners.{}.id", &handle), l.get_id());
        });
        self.steps.iter().for_each(|step| {
            let step = step.lock().unwrap();
            let handle = step.get_handle().to_string();
            pipeline_data.insert(format!("steps.{}.handle", &handle), handle.clone());
            pipeline_data.insert(format!("steps.{}.id", &handle), step.get_id());
        });

        for listener_mtx in self.listeners.iter_mut() {
            let mut listener = listener_mtx.lock().unwrap();
            let module_handle = listener.component.handle;
            listener.component.args.iter()
                .for_each(|(k,v) | listener.module.set_param(module_handle, k, v));
            pipeline_data.iter().for_each(|(k, v)| {
                listener.module.set_param(module_handle, format!("pipeline.{}", k), v.clone())
            });
            listener.configure(ModuleListenerConfigureArgs{
                module_handle: std_types::Uint::try_from(module_handle).unwrap(),
            })?;
        }
        Ok(())
    }

    /// Start senders and receivers
    pub fn start_senders_receivers(&self) -> Result<(), String> {
        let mut senders = SENDERS.lock().unwrap();

        let (m_tx, m_rx) = channel::<SystemMessage>();
        if let Err(_) = SYSTEM_MESSAGES.set(m_tx) {
            return Err(String::from("Failed to initialize a system message channel"))
        };

        start_system_command_thread(m_rx);

        let listeners: Vec<Listener> = self.listeners
            .iter()
            .map(|l| l.lock().unwrap().clone())
            .collect();

        for i in 0..self.steps.len() - 1 {
            let i_sender = i;
            let i_receiver = i_sender + 1;
            let step_sender_arc = self.steps
                .get(i_sender).unwrap().clone();
            let step_receiver_arc = self.steps
                .get(i_receiver).unwrap().clone();

            let i_sender_ffi = u32::try_from(i_sender).unwrap();

            // Store a pointer to Free Record function in static context
            FREE_BUF.lock().unwrap().insert(i_sender_ffi, *step_sender_arc.lock().unwrap().module.free_record_ptr.clone());
            // Record channels
            let (tx, rx) = channel::<Record>();
            senders.insert(i_sender_ffi, tx);

            start_reader_thread(step_sender_arc, step_receiver_arc, rx, listeners.clone());
        }

        Ok(())
    }

    /// Starts the data processing routines inside each step
    pub fn start_steps(&self) -> Result<(), String> {
        info!("Starting steps...");
        for step_mtx in &self.listeners {
            let step = step_mtx.lock().unwrap();
            let module_handle = step.component.handle;
            match step.module.start(module_handle.into()) {
                Ok(_) => debug!("Started event listener '{}'", step.component.id),
                Err(msg) => {
                    return Err(format!("Failed to start event listener '{}': {}", step.component.id, msg));
                }
            }
        }
        for step_mtx in &self.steps {
            let step = step_mtx.lock().unwrap();
            let module_handle = step.component.handle;
            match step.module.start(module_handle.into()) {
                Ok(_) => debug!("Started pipeline step '{}'", step.component.id),
                Err(msg) => {
                    return Err(format!("Failed to start pipeline step '{}': {}", step.component.id, msg));
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
            .map(|step| match step.lock().unwrap().component.is_terminated() { true => 1, false => 0 })
            .fold(0, |acc, e| acc + e );

        steps_terminated < steps_total
    }

    pub fn get_step_by_handle_mut(&self, handle: usize) -> Option<Arc<Mutex<PipelineStep>>> {
        for h in &self.steps {
            if h.lock().unwrap().component.handle == handle {
                return Some(h.clone())
            }
        }
        None
    }

    pub fn trigger_termination(&self) {
        let first_step = self.steps
            .first().unwrap()
            .lock().unwrap();
        first_step.shutdown();
    }
}

impl TryFrom<(&PipelineDefinition, &LoadedLibraries)> for Pipeline {
    fn try_from(value: (&PipelineDefinition, &LoadedLibraries)) -> Result<Self, Self::Error> {
        let (definition, loaded_libs) = value;
        // Validate references to modules
        let mut pipeline = Pipeline::new();
        pipeline.description = definition.description.clone();

        for step_def in &definition.steps {
            if loaded_libs.pipeline.get(&step_def.handler).is_none() {
                return Err(format!("Module not found: {}", &step_def.handler));
            }
        }

        let mut step_index: usize = 0;
        pipeline.steps = definition
            .steps
            .iter()
            // .enumerate()
            .map(|step_def| {
                let s = PipelineStep::from_module(
                    loaded_libs.pipeline.get(&step_def.handler).unwrap().clone(),
                    step_index, step_def.args.clone());
                step_index += 1;
                Arc::new(Mutex::new(s))
            })
            .collect();
        pipeline.listeners = definition
            .listeners
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            // .enumerate()
            .map(|step_def| {
                let s = Listener::from_module(
                    loaded_libs.listeners.get(&step_def.handler).unwrap().clone(),
                    step_index, step_def.args.clone());
                step_index += 1;
                Arc::new(Mutex::new(s))
            })
            .collect();
        pipeline.validate()?;

        Ok(pipeline)
    }

    type Error = String;
}