use std::future::Future;
use std::time::Duration;

use anyhow::Context;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

use crate::cli::Config;
use crate::models::{CityId, CurrentWeather};

/// Api docs: <https://openweathermap.org/current>
const V25_ENDPOINT: &str = "https://api.openweathermap.org/data/2.5/weather";

#[derive(Debug, thiserror::Error)]
pub enum ApiCallError {
    #[error("error building request: {0}")]
    RequestBuild(#[from] reqwest::Error),
    #[error("unexpected response. Status: {status}, Body: {body}")]
    InvalidResponseStatus {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("error parsing response body: {0}")]
    ResponseParse(#[from] serde_json::Error),
}

pub struct ConfigClient {
    pub config: Config,
    client: reqwest::Client,
    api_key: String,
    city_ids: Vec<CityId>,
}

impl ConfigClient {
    pub fn new(config: &Config) -> Result<Self, anyhow::Error> {
        let client = reqwest::Client::builder()
            .https_only(true)
            .build()
            .context("error building HTTP client")?;
        let city_ids = config.city_ids.iter().map(|id| CityId::new(*id)).collect();
        Ok(Self {
            config: config.clone(),
            client,
            api_key: config.api_key.clone(),
            city_ids,
        })
    }

    pub fn fetch_weather(
        &self,
        id: &CityId,
    ) -> impl Future<Output = Result<CurrentWeather, ApiCallError>> + '_ {
        let url = format!(
            "{V25_ENDPOINT}?units=metric&lang=en&appid={}&id={}",
            self.api_key, id.id
        );
        log::debug!(
            "Fetch weather at URL with masked key: {}",
            mask(&url, &self.api_key)
        );
        async move {
            let response = self.client.get(&url).send().await?;
            let status = response.status();
            if status == reqwest::StatusCode::OK {
                let weather = response.json::<CurrentWeather>().await?;
                Ok(weather)
            } else {
                let body = response.text().await.unwrap_or_default();
                Err(ApiCallError::InvalidResponseStatus { status, body })
            }
        }
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
                tokio::time::timeout(Duration::from_secs(5), self.fetch_weather(id)).await
            })
    }
}

/// Replace all occurrences of the API key in a string with a masked version.
fn mask(s: &str, api_key: &str) -> String {
    if api_key.is_empty() {
        return s.to_string();
    }
    let masked = if api_key.len() > 3 {
        format!("{}****", &api_key[..3])
    } else {
        "****".to_string()
    };
    s.replace(api_key, &masked)
}
