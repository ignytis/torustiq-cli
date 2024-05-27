use std::{fs, io};
use std::error::Error;

use clap::{arg, command, Parser};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct PipelineStep {
    name: String,
    handler: String,
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct PipelineDefinition {
    steps: Vec<PipelineStep>,
}


/// Starts a data processing pipeline from provided config
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// A YAML file to read the pipeline structure from
    #[arg(short, long, default_value="pipeline.yaml")]
    pipeline_file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let cfg: String = match fs::read_to_string(&args.pipeline_file) {
        Ok(c) => c,
        Err(e) => return err(format!("Cannot open the pipeline file: '{}'. {}", args.pipeline_file, e)),
    };
    let _cfg: PipelineDefinition = match serde_yaml::from_str(cfg.as_str()) {
        Ok(c) => c,
        Err(e) => return err(format!("Cannot parse the pipeline: '{}'. {}", args.pipeline_file, e)),
    };
    Ok(())
}

fn err(message: String) -> Result<(), Box<dyn Error>> {
    Err(Box::new(io::Error::new(io::ErrorKind::Other, message)))
}