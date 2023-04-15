use std::future::Future;
use std::time::Duration;

use openweathermap_client::error::ApiCallError;
use openweathermap_client::models::{CityId, CurrentWeather};
use openweathermap_client::Client;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

use crate::cli::Config;

pub struct ConfigClient {
    pub config: Config,
    client: Client,
    city_ids: Vec<CityId>,
}

impl ConfigClient {
    pub fn new(config: &Config) -> Result<Self, anyhow::Error> {
        let client = config.api_client()?;
        let city_ids = config.city_ids.iter().map(|id| CityId::new(*id)).collect();

        Ok(Self {
            config: config.clone(),
            client,
            city_ids,
        })
    }

    #[allow(clippy::needless_lifetimes)]
    pub fn get_weather<'a>(
        &'a self,
    ) -> impl Stream<
        Item = impl Future<
            Output = Result<Result<CurrentWeather, ApiCallError>, tokio::time::error::Elapsed>,
        > + 'a,
    > + 'a {
        tokio_stream::iter(self.city_ids.iter())
            .throttle(std::time::Duration::from_secs(1))
            .map(move |id| async move {
                log::info!("Fetching weather for {}", id);
                tokio::time::timeout(Duration::from_secs(5), self.client.fetch_weather(id)).await
            })
    }
}
