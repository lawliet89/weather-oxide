mod cli;

use clap::Parser;
use openweathermap_client::models::{City, Weather};
use tokio_stream::StreamExt;

/// Gets the temperature in °C and description of the weather in Paris right now
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let cli = cli::Cli::parse();
    log::info!("Reading configuration file...");
    let config = cli.config()?;

    let client = config.client()?;
    let weather = client.get_weather();

    tokio::pin!(weather);

    while let Some(city) = weather.next().await {
        println!("{:?}", city.unwrap().await);
        // let weather = city.unwrap();
    }

    // println!("{:?}", weather);
    // let reading = client.fetch_weather(&City::new("Paris", "FR")).await?;

    // println!(
    //     "The temperature and weather in France in French is {}, {}",
    //     reading.main.temp, reading.weather[0].description
    // );
    Ok(())
}
