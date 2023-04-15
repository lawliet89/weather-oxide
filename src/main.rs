mod cli;
mod client;
mod data;

use clap::Parser;
use client::ConfigClient;
use tokio_stream::StreamExt;

async fn get_weather(client: &ConfigClient) {
    let weather = client.get_weather();

    tokio::pin!(weather);

    while let Some(result) = weather.next().await {
        match result.await {
            Ok(call) => match call {
                Ok(weather) => {
                    log::debug!("{:?}", weather);
                    let record = data::LegacyRecord::from(weather);
                    match record.append(
                        &client.config.output.directory,
                        &client.config.output.extension,
                        client.config.output.delimiter,
                    ) {
                        Ok(()) => log::info!("wrote entry for {}", record.city),
                        Err(e) => log::error!("error appending entry {e}"),
                    }
                }
                Err(e) => log::error!("error calling api: {e}"),
            },
            Err(e) => {
                log::error!("timeout fetching {e}");
            }
        }
    }
}

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

    loop {
        get_weather(&client).await;
        log::info!("Sleeping for 30 minutes");
        tokio::time::sleep(std::time::Duration::from_secs(30 * 60)).await;
    }
}
