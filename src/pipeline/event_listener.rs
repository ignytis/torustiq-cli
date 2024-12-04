use std::collections::HashMap;
use std::sync::Arc;

use torustiq_common::ffi::types::module::ModulePipelineStepConfigureArgs;

use crate::{
    modules::event_listener::EventListenerModule,
    pipeline::{PipelineComponent, PipelineComponentState},
};

/// An event listener for pipeline events
pub struct EventListener {
    pub component: PipelineComponent,
    /// A reference to module library
    pub module: Arc<EventListenerModule>,
}

impl EventListener {
    /// Initializes a step from module (=dynamic library).
    /// Index is a step index in pipeline. Needed to format a unique step ID
    pub fn from_module(module: Arc<EventListenerModule>, handle: usize, args: Option<HashMap<String, String>>) -> EventListener {
        EventListener {
            component: PipelineComponent {
                args: args.unwrap_or(HashMap::new()),
                handle,
                id: format!("evt_listener_{}_{}", handle, module.get_info().id),
                state: PipelineComponentState::Created,
            },
            module,
        }
    }

    pub fn configure(&mut self, args: ModulePipelineStepConfigureArgs) -> Result<(), String> {
        let _ = self.module.configure_step(args)?;
        self.component.state = PipelineComponentState::Configured;
        Ok(())
    }

    pub fn shutdown(&self) {
        self.module.shutdown(self.component.handle);
    }
}