use std::{
    collections::HashMap, sync::Arc,
};

use log::debug;
use torustiq_common::ffi::types::module::ModuleStepConfigureArgs;

use crate::module::Module;


/// State of step
#[derive(PartialEq)]
pub enum PipelineStepState {
    /// Step is just created, but not configured and not ready to run
    Created,
    /// Step is configured and ready to run
    Configured,
    /// Step is running
    Running,
    /// Step is terminated
    Terminated,
}

/// A single step in pipeline
pub struct PipelineStep {
    /// Module-specific arguments - credentials, formatting rules, etc
    pub args: HashMap<String, String>,
    /// This handle is passed to modules in order to identity a step
    pub handle: usize,
    /// A human-readable identifier
    pub id: String,
    /// A reference to module library
    pub module: Arc<Module>,
    /// State of step
    pub state: PipelineStepState,
}

impl PipelineStep {
    /// Initializes a step from module (=dynamic library).
    /// Index is a step index in pipeline. Needed to format a unique step ID
    pub fn from_module(module: Arc<Module>, handle: usize, args: Option<HashMap<String, String>>) -> PipelineStep {
        PipelineStep {
            args: args.unwrap_or(HashMap::new()),
            handle,
            id: format!("step_{}_{}", handle, module.module_info.id),
            module,
            state: PipelineStepState::Created,
        }
    }

    pub fn configure(&mut self, args: ModuleStepConfigureArgs) -> Result<(), String> {
        let _ = self.module.configure_step(args)?;
        self.state = PipelineStepState::Configured;
        Ok(())
    }

    pub fn set_state_terminated(&mut self) {
        debug!("Marking the pipeline step '{}' as terminated.", self.id);
        self.state = PipelineStepState::Terminated
    }

    pub fn is_terminated(&self) -> bool {
        self.state == PipelineStepState::Terminated
    }

    pub fn shutdown(&self) {
        self.module.shutdown(self.handle);
    }
}