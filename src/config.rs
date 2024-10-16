use std::collections::HashMap;

use serde::{Serialize, Deserialize};

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
    pub description: Option<String>,
    pub steps: Vec<PipelineStepDefinition>,
}