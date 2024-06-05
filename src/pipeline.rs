use std::{collections::HashMap, str};

use serde::{Serialize, Deserialize};

use torustiq_common::types::module::{IoKind, Module};

/// A step in pipeline: source, destination, transformation, etc
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PipelineStepDefinition {
    pub name: String,
    pub handler: String,
}

/// A pipeline definition. Contains multiple steps
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PipelineDefinition {
    pub steps: Vec<PipelineStepDefinition>,
}

pub struct Pipeline {
    pub steps: Vec<Module>,
}

impl Pipeline {
    fn new() -> Pipeline {
        Pipeline {
            steps: Vec::new(),
        }
    }

    pub fn build(definition: &PipelineDefinition, modules: &HashMap<String, Module>) -> Result<Pipeline, String> {
        // Validate references to modules
        for step_def in &definition.steps {
            if modules.get(&step_def.handler).is_none() {
                return Err(format!("Module not found: {}", &step_def.handler));
            }
        }

        let mut pipeline = Pipeline::new();
        pipeline.steps = definition
            .steps
            .iter().map(|step_def| modules.get(&step_def.handler).unwrap().clone())
            .collect();

        // Validate: check if the first step has external (=user defined) input and the last step has external output
        // Internal input/outputs are passed between steps
        if let Some(s) = pipeline.steps.first() {
            if s.input_kind != IoKind::External {
                return Err(format!("Input of the first step is not 'external'. The first step must have an external input"))
            }
        }
        if let Some(s) = pipeline.steps.last() {
            if s.output_kind != IoKind::External {
                return Err(format!("Output of the last step is not 'external'. The last step must have an external output"))
            }
        }

        let steps_len = pipeline.steps.len();
        if steps_len < 2 {
            return Err(format!("Pipeline must have at least two steps. Teh actual number of steps: {}", steps_len))
        }

        for i in 0..pipeline.steps.len()-1 {
            // output of step[n] is input of step[n+1]
            let step_output = pipeline.steps.get(i).unwrap();
            let step_input = pipeline.steps.get(i+1).unwrap();

            if step_output.output_kind != step_input.input_kind {
                return Err(format!("Kinds of data mismatch between steps {} and {}: {:?} vs {:?}", i+1, i+2, step_output.output_kind, step_input.input_kind))
            }
        }
        Ok(pipeline)
    }
}