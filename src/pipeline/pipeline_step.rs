use std::{
    collections::HashMap, sync::Arc,
};

use torustiq_common::ffi::types::module::ModulePipelineConfigureArgs;

use crate::{
    modules::pipeline::PipelineModule,
    pipeline::{PipelineComponent, PipelineComponentState},
};

/// A single step in pipeline
pub struct PipelineStep {
    /// Base pipeline component attributes
    pub component: PipelineComponent,
    /// A reference to module library
    pub module: Arc<PipelineModule>,
}

impl PipelineStep {
    /// Initializes a step from module (=dynamic library).
    /// Index is a step index in pipeline. Needed to format a unique step ID
    pub fn from_module(module: Arc<PipelineModule>, handle: usize, args: Option<HashMap<String, String>>) -> PipelineStep {
        PipelineStep {
            component: PipelineComponent {
                args: args.unwrap_or(HashMap::new()),
                handle,
                id: format!("step_{}_{}", handle, module.get_info().id),
                state: PipelineComponentState::Created,
            },
            module,
        }
    }

    pub fn configure(&mut self, args: ModulePipelineConfigureArgs) -> Result<(), String> {
        let _ = self.module.configure_step(args)?;
        self.component.state = PipelineComponentState::Configured;
        Ok(())
    }

    pub fn shutdown(&self) {
        self.module.shutdown(self.component.handle);
    }
}