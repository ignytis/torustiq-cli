use std::collections::HashMap;
use std::sync::Arc;

use torustiq_common::ffi::types::module::ModuleListenerConfigureArgs;

use crate::{
    modules::listener::ListenerModule,
    pipeline::{PipelineComponent, PipelineComponentState},
};

/// An event listener for pipeline events
pub struct Listener {
    pub component: PipelineComponent,
    /// A reference to module library
    pub module: Arc<ListenerModule>,
}

impl Listener {
    /// Initializes a step from module (=dynamic library).
    /// Index is a step index in pipeline. Needed to format a unique step ID
    pub fn from_module(module: Arc<ListenerModule>, handle: usize, args: Option<HashMap<String, String>>) -> Listener {
        Listener {
            component: PipelineComponent {
                args: args.unwrap_or(HashMap::new()),
                handle,
                id: format!("evt_listener_{}_{}", handle, module.get_info().id),
                state: PipelineComponentState::Created,
            },
            module,
        }
    }

    pub fn configure(&mut self, args: ModuleListenerConfigureArgs) -> Result<(), String> {
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
}