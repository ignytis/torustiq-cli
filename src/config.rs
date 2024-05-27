use std::collections::HashMap;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Http {
    pub host: String,
    pub port: u16,
    pub num_workers: usize
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub http: Http,
    pub kafka: HashMap<String, String>,
}