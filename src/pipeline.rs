use std::{collections::HashMap, sync::Arc, str};

use serde::{Serialize, Deserialize};

use crate::module::Module;

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
    pub steps: Vec<PipelineStep>,
}

impl Pipeline {
    pub fn new() -> Pipeline {
        Pipeline {
            steps: Vec::new(),
        }
    }

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