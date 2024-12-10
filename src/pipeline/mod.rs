use std::collections::HashMap;

use log::debug;

pub mod listener;
pub mod pipeline;
pub mod pipeline_step;

/// State of step
#[derive(PartialEq)]
pub enum PipelineComponentState {
    /// Component is just created, but not configured and not ready to run
    Created,
    /// Component is configured and ready to run
    Configured,
    /// Component is running
    Running,
    /// Component is terminated
    Terminated,
}

/// Common properties for pipeline components
/// Pipeline component is whatever part of pipeline which utilized module features:
/// - Pipeline Step
/// - Event listener
pub struct PipelineComponent {
    /// Module-specific arguments - credentials, formatting rules, etc
    pub args: HashMap<String, String>,
    /// This handle is passed to modules in order to identify a step
    pub handle: usize,
    /// A human-readable identifier
    pub id: String,
    /// State of step
    pub state: PipelineComponentState,
}

impl PipelineComponent {
    pub fn set_state_terminated(&mut self) {
        debug!("Marking the pipeline step '{}' as terminated.", self.id);
        self.state = PipelineComponentState::Terminated
    }

    pub fn is_terminated(&self) -> bool {
        self.state == PipelineComponentState::Terminated
    }
}