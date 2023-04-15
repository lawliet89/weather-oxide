mod cli;

use clap::Parser;

use openweathermap_client::models::{City, UnitSystem};
use openweathermap_client::{Client, ClientOptions};

/// Gets the temperature in °C and description of the weather in Paris right now
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let cli = cli::Cli::parse();
    let options = ClientOptions {
        units: UnitSystem::Metric,
        language: "en".to_string(),
        api_key: cli.get_token()?,
    };
    let client = Client::new(options)?;
    let reading = client.fetch_weather(&City::new("Paris", "FR")).await?;

    println!(
        "The temperature and weather in France in French is {}, {}",
        reading.main.temp, reading.weather[0].description
    );
    Ok(())
}
