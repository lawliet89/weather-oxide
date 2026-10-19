use std::future::Future;
use std::time::Duration;

use anyhow::Context;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

use crate::cli::Config;
use crate::models::CurrentWeather;

/// Api docs: <https://openweathermap.org/current>
const V25_ENDPOINT: &str = "https://api.openweathermap.org/data/2.5/weather";

#[derive(Debug, thiserror::Error)]
pub enum ApiCallError {
    #[error("error sending request: {0}")]
    RequestSend(#[source] reqwest::Error),
    #[error("error building request: {0}")]
    RequestBuild(#[source] reqwest::Error),
    #[error("unexpected response. Status: {status}, Body: {body}")]
    InvalidResponseStatus {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("error parsing response body: {0}")]
    ResponseParse(#[source] reqwest::Error),
}

pub struct ConfigClient {
    pub config: Config,
    client: reqwest::Client,
    city_ids: Vec<u32>,
}

impl ConfigClient {
    pub fn new(config: &Config) -> Result<Self, anyhow::Error> {
        let client = reqwest::Client::builder()
            .https_only(true)
            .build()
            .context("error building HTTP client")?;
        Ok(Self {
            config: config.clone(),
            client,
            city_ids: config.city_ids.clone(),
        })
    }

    pub fn fetch_weather(
        &self,
        id: u32,
    ) -> Result<impl Future<Output = Result<CurrentWeather, ApiCallError>> + '_, ApiCallError> {
        let id_string = id.to_string();
        let request = self
            .client
            .get(V25_ENDPOINT)
            .query(&[
                ("units", "metric"),
                ("lang", "en"),
                ("appid", self.config.api_key.as_str()),
                ("id", id_string.as_str()),
            ])
            .build()
            // The URL contains the API key; strip it from the error so it
            // can never leak into logs.
            .map_err(|e| ApiCallError::RequestBuild(e.without_url()))?;
        log::debug!(
            "Fetch weather at URL with masked key: {}",
            mask(request.url().as_str(), &self.config.api_key)
        );
        Ok(async move {
            let response = self
                .client
                .execute(request)
                .await
                // The URL contains the API key; strip it from the error so
                // it can never leak into logs.
                .map_err(|e| ApiCallError::RequestSend(e.without_url()))?;
            let status = response.status();
            if status == reqwest::StatusCode::OK {
                let weather = response
                    .json::<CurrentWeather>()
                    .await
                    .map_err(|e| ApiCallError::ResponseParse(e.without_url()))?;
                Ok(weather)
            } else {
                let body = response.text().await.unwrap_or_default();
                Err(ApiCallError::InvalidResponseStatus { status, body })
            }
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
        tokio_stream::iter(self.city_ids.iter().copied())
            .throttle(std::time::Duration::from_secs(1))
            .map(move |id| async move {
                log::info!("Fetching weather for {}", id);
                let fetch = self.fetch_weather(id);
                match fetch {
                    // Request could not be built; no future to await.
                    Err(e) => Ok(Err(e)),
                    Ok(future) => tokio::time::timeout(Duration::from_secs(5), future).await,
                }
            })
    }
}

/// Replace all occurrences of the API key in a string with a masked version.
fn mask(s: &str, api_key: &str) -> String {
    if api_key.is_empty() {
        return s.to_string();
    }
    let masked = if api_key.chars().count() > 3 {
        format!("{}****", api_key.chars().take(3).collect::<String>())
    } else {
        "****".to_string()
    };
    s.replace(api_key, &masked)
}
