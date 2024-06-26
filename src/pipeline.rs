use std::{collections::HashMap, sync::Arc, str};

use serde::{Serialize, Deserialize};

use torustiq_common::ffi::types::module::IoKind;

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

    pub fn from_definition(definition: &PipelineDefinition, modules: &HashMap<String, Arc<Module>>) -> Self {
        // Validate references to modules
        let mut pipeline = Pipeline::new();
        for step_def in &definition.steps {
            if modules.get(&step_def.handler).is_none() {
                panic!("Module not found: {}", &step_def.handler);
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
        pipeline.validate();

        pipeline
    }

    fn validate(&self) {
        // Validate: check if the first step has external (=user defined) input and the last step has external output
        // Internal input/outputs are passed between steps
        if let Some(s) = self.steps.first() {
            if s.module.module_info.input_kind != IoKind::External {
                panic!("Input of the first step is not 'external'. The first step must have an external input")
            }
        }
        if let Some(s) = self.steps.last() {
            if s.module.module_info.output_kind != IoKind::External {
                panic!("Output of the last step is not 'external'. The last step must have an external output")
            }
        }

        let steps_len = self.steps.len();
        if steps_len < 2 {
            panic!("Pipeline must have at least two steps. The actual number of steps: {}", steps_len)
        }

        for i in 0..self.steps.len()-1 {
            // output of step[n] is input of step[n+1]
            let step_output = self.steps.get(i).unwrap();
            let step_input = self.steps.get(i+1).unwrap();

            if step_output.module.module_info.output_kind != step_input.module.module_info.input_kind {
                panic!("Kinds of data mismatch between steps {} and {}: {:?} vs {:?}", i+1, i+2,
                    step_output.module.module_info.output_kind, step_input.module.module_info.input_kind)
            }
        }
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