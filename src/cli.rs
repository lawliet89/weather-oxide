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

#[cfg(test)]
mod tests {
    use super::{Cli, Config};

    #[test]
    fn parses_full_config() {
        let config: Config = hcl::from_str(
            r#"
            api_key = "secret"
            city_ids = [1, 2, 3]
            output {
              directory = "/data"
              delimiter = ";"
              extension = "txt"
            }
            "#,
        )
        .unwrap();
        assert_eq!(config.api_key, "secret");
        assert_eq!(config.city_ids, vec![1, 2, 3]);
        assert_eq!(config.output.directory, "/data");
        assert_eq!(config.output.delimiter, ';');
        assert_eq!(config.output.extension, "txt");
    }

    #[test]
    fn output_defaults_to_csv_with_comma() {
        let config: Config = hcl::from_str(
            r#"
            api_key = "secret"
            city_ids = []
            output {
              directory = "./output"
            }
            "#,
        )
        .unwrap();
        assert!(config.city_ids.is_empty());
        assert_eq!(config.output.delimiter, ',');
        assert_eq!(config.output.extension, "csv");
    }

    #[test]
    fn missing_api_key_is_an_error() {
        let result: Result<Config, _> = hcl::from_str(
            r#"
            city_ids = [1]
            output {
              directory = "./output"
            }
            "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn loads_bundled_template() {
        let cli = Cli {
            config_file: concat!(env!("CARGO_MANIFEST_DIR"), "/config.template.hcl").to_string(),
        };
        let config = cli.config().unwrap();
        assert!(!config.api_key.is_empty());
        assert!(config.city_ids.contains(&1880252));
        assert_eq!(config.output.directory, "./output");
        assert_eq!(config.output.delimiter, ',');
        assert_eq!(config.output.extension, "csv");
    }

    #[test]
    fn missing_config_file_mentions_path() {
        let cli = Cli {
            config_file: "/nonexistent/weather-oxide.hcl".to_string(),
        };
        let err = cli.config().unwrap_err();
        assert!(
            err.to_string().contains("/nonexistent/weather-oxide.hcl"),
            "{err}"
        );
    }
}
