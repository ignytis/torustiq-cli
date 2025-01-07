use std::{
    collections::HashMap, sync::Arc,
};

use torustiq_common::ffi::types::{
    module as module_types,
    std_types,
};

use crate::{
    modules::pipeline::PipelineModule,
    pipeline::{PipelineComponent, PipelineComponentState},
};

/// A single step in pipeline
#[derive(Clone)]
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

    pub fn configure(&mut self, args: module_types::ModulePipelineConfigureArgs) -> Result<(), String> {
        let _ = self.module.configure(args)?;
        self.component.state = PipelineComponentState::Configured;
        Ok(())
    }

    pub fn shutdown(&self) {
        self.module.shutdown(self.component.handle);
    }

    pub fn get_id(&self) -> String {
        self.component.id.clone()
    }

    pub fn get_handle(&self) -> usize {
        self.component.handle
    }

    pub fn ffi_free_char(&self, c: std_types::ConstCharPtr) {
        (self.module.base.free_char_ptr)(c);
    }

    pub fn ffi_process_record(&self, record: module_types::Record, handle: module_types::ModuleHandle) -> module_types::ModulePipelineProcessRecordFnResult {
        (self.module.process_record_ptr)(handle, record)

    }

    pub fn ffi_shutdown(&self, handle: module_types::ModuleHandle) {
        (self.module.base.shutdown_ptr)(handle);
    }
}