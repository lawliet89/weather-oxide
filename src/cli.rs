use std::fs::File;
use std::future::Future;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use openweathermap_client::error::ApiCallError;
use openweathermap_client::models::{CityId, CurrentWeather, UnitSystem};
use openweathermap_client::{Client, ClientOptions};
use serde::{Deserialize, Serialize};
use tokio_stream::Stream;
use tokio_stream::StreamExt;

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
    directory: String,
    #[serde(default = "default_delimiter")]
    delimiter: char,
    #[serde(default = "default_extension")]
    extension: String,
}

impl Config {
    pub fn client(&self) -> Result<ConfigClient, anyhow::Error> {
        ConfigClient::new(self)
    }

    fn api_client(&self) -> Result<Client, anyhow::Error> {
        let options = ClientOptions {
            units: UnitSystem::Metric,
            language: "en".to_string(),
            api_key: self.api_key.clone(),
        };
        let client = Client::new(options).with_context(|| "error making OpenWeatherMap Client")?;
        Ok(client)
    }
}

pub struct ConfigClient {
    config: Config,
    client: Client,
    city_ids: Vec<CityId>,
}

impl ConfigClient {
    fn new(config: &Config) -> Result<Self, anyhow::Error> {
        let client = config.api_client()?;
        let city_ids = config.city_ids.iter().map(|id| CityId::new(*id)).collect();

        Ok(Self {
            config: config.clone(),
            client,
            city_ids,
        })
    }

    pub fn get_weather<'a>(&'a self) -> impl Stream + 'a {
        tokio_stream::iter(self.city_ids.iter())
            .throttle(std::time::Duration::from_secs(1))
            .map(|id|{
                log::info!("Fetching weather for {}", id);
                self.client.fetch_weather(id)
            })
            .timeout(Duration::from_secs(5))
    }
}

fn default_delimiter() -> char {
    ','
}

fn default_extension() -> String {
    "csv".to_string()
}
