use std::fs::File;

use crate::client::ConfigClient;
use anyhow::Context;
use clap::Parser;
use serde::{Deserialize, Serialize};

/// Fetch weather information periodically
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to Configuration File
    #[arg(env)]
    config_file: String,
}

impl Cli {
    pub fn config(&self) -> Result<Config, anyhow::Error> {
        let mut config_file = File::open(&self.config_file)
            .with_context(|| format!("error opening config file {}", self.config_file))?;
        let config = hcl::from_reader(&mut config_file).with_context(|| "error reading HCL")?;
        Ok(config)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub city_ids: Vec<u32>,
    pub output: Output,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub directory: String,
    #[serde(default = "default_delimiter")]
    pub delimiter: char,
    #[serde(default = "default_extension")]
    pub extension: String,
}

impl Config {
    pub fn client(&self) -> Result<ConfigClient, anyhow::Error> {
        ConfigClient::new(self)
    }
}

fn default_delimiter() -> char {
    ','
}

fn default_extension() -> String {
    "csv".to_string()
}
