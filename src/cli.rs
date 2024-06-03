use clap::{arg, command, Parser};

/// Starts a data processing pipeline from provided config
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// A YAML file to read the pipeline structure from
    #[arg(short, long, default_value="pipeline.yaml")]
    pub pipeline_file: String,

    /// A YAML file to read the pipeline structure from
    #[arg(short, long, default_value="modules")]
    pub module_dir: String,
}

impl CliArgs {
    pub fn do_parse() -> CliArgs {
        CliArgs::parse()
    }
}