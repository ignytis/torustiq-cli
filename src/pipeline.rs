use std::{collections::HashMap, str};

use serde::{Serialize, Deserialize};

use torustiq_common::types::module::Module;

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

    pub fn build(definition: &PipelineDefinition, modules: &HashMap<String, Module>) -> Pipeline {
        let mut pipeline = Pipeline::new();

        pipeline
    }
}