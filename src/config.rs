use std::collections::{HashMap, HashSet};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ModuleDefinition {
    pub name: String,
    pub handler: String,
    pub args: Option<HashMap<String, String>>,
}

/// A pipeline definition. Contains multiple steps
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PipelineDefinition {
    pub description: Option<String>,
    /// Steps: source, destination, transformations.
    /// The data processing happens here
    pub steps: Vec<ModuleDefinition>,
    /// Event listeners handle application events: processed records, failures, etc
    pub listeners: Option<Vec<ModuleDefinition>>,
}

impl PipelineDefinition {
    /// Returns a vector of module ID-s which are used by pipeline
    pub fn get_module_ids_in_use(&self) -> Vec<String> {
        let required_module_ids: Vec<String> = { // collect all module IDs from pipeline definition
            let listener_modules = match &self.listeners {
                Some(l) => l,
                None => &Vec::new(),
            };
            let step_modules = &self.steps;
            step_modules
                .into_iter()
                .chain(listener_modules.into_iter())
                .map(|step| step.handler.clone())
                .collect::<HashSet<_>>() // deduplicate
                .into_iter()
                .collect()
        };

        required_module_ids
    }
}