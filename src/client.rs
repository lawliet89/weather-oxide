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
    endpoint: String,
}

impl ConfigClient {
    pub fn new(config: &Config) -> Result<Self, anyhow::Error> {
        Self::build(config, V25_ENDPOINT, true)
    }

    /// Point the client at a different endpoint, e.g. a local mock server.
    /// HTTPS is not enforced so plain-HTTP test servers work.
    #[cfg(test)]
    fn with_endpoint(config: &Config, endpoint: &str) -> Result<Self, anyhow::Error> {
        Self::build(config, endpoint, false)
    }

    fn build(config: &Config, endpoint: &str, https_only: bool) -> Result<Self, anyhow::Error> {
        let client = reqwest::Client::builder()
            .https_only(https_only)
            .build()
            .context("error building HTTP client")?;
        Ok(Self {
            config: config.clone(),
            client,
            endpoint: endpoint.to_string(),
        })
    }

    pub async fn fetch_weather(&self, id: u32) -> Result<CurrentWeather, ApiCallError> {
        let id_string = id.to_string();
        let request = self
            .client
            .get(&self.endpoint)
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
    }

    #[allow(clippy::needless_lifetimes)]
    pub fn get_weather<'a>(
        &'a self,
    ) -> impl Stream<
        Item = impl Future<
            Output = Result<Result<CurrentWeather, ApiCallError>, tokio::time::error::Elapsed>,
        > + 'a,
    > + 'a {
        tokio_stream::iter(self.config.city_ids.iter().copied())
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
    let masked = if api_key.chars().count() > 3 {
        format!("{}****", api_key.chars().take(3).collect::<String>())
    } else {
        "****".to_string()
    };
    s.replace(api_key, &masked)
}

#[cfg(test)]
mod tests {
    use tokio_stream::StreamExt;
    use wiremock::matchers::{method, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{mask, ApiCallError, ConfigClient};
    use crate::cli::{Config, Output};
    use crate::models::tests::SAMPLE;

    fn config(city_ids: Vec<u32>) -> Config {
        Config {
            api_key: "test-key".to_string(),
            city_ids,
            output: Output {
                directory: "./output".to_string(),
                delimiter: ',',
                extension: "csv".to_string(),
            },
        }
    }

    fn sample_response() -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_raw(SAMPLE, "application/json")
    }

    #[test]
    fn production_client_uses_https_endpoint() {
        let client = ConfigClient::new(&config(vec![])).unwrap();
        assert_eq!(client.endpoint, super::V25_ENDPOINT);
        assert!(client.endpoint.starts_with("https://"));
    }

    #[tokio::test]
    async fn https_only_client_rejects_plain_http() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(sample_response())
            .expect(0)
            .mount(&server)
            .await;

        let client = ConfigClient::build(&config(vec![]), &server.uri(), true).unwrap();
        let err = client.fetch_weather(1).await.unwrap_err();
        assert!(matches!(err, ApiCallError::RequestSend(_)), "{err:?}");
    }

    #[tokio::test]
    async fn fetch_weather_sends_expected_query() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(query_param("appid", "test-key"))
            .and(query_param("id", "1880252"))
            .and(query_param("units", "metric"))
            .and(query_param("lang", "en"))
            .respond_with(sample_response())
            .expect(1)
            .mount(&server)
            .await;

        let client = ConfigClient::with_endpoint(&config(vec![]), &server.uri()).unwrap();
        let weather = client.fetch_weather(1880252).await.unwrap();
        assert_eq!(weather.name, "Singapore");
        assert_eq!(weather.id, 1880252);
    }

    #[tokio::test]
    async fn fetch_weather_reports_non_200_with_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Invalid API key"))
            .mount(&server)
            .await;

        let client = ConfigClient::with_endpoint(&config(vec![]), &server.uri()).unwrap();
        match client.fetch_weather(1).await {
            Err(ApiCallError::InvalidResponseStatus { status, body }) => {
                assert_eq!(status, reqwest::StatusCode::UNAUTHORIZED);
                assert_eq!(body, "Invalid API key");
            }
            other => panic!("expected InvalidResponseStatus, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fetch_weather_reports_unparseable_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_raw("{}", "application/json"))
            .mount(&server)
            .await;

        let client = ConfigClient::with_endpoint(&config(vec![]), &server.uri()).unwrap();
        let err = client.fetch_weather(1).await.unwrap_err();
        assert!(matches!(err, ApiCallError::ResponseParse(_)), "{err:?}");
    }

    #[tokio::test]
    async fn fetch_weather_errors_do_not_leak_api_key() {
        // Nothing is listening on this port, so the request fails to send.
        let client = ConfigClient::with_endpoint(&config(vec![]), "http://127.0.0.1:1").unwrap();
        let err = client.fetch_weather(1).await.unwrap_err();
        assert!(matches!(err, ApiCallError::RequestSend(_)), "{err:?}");
        assert!(!format!("{err} {err:?}").contains("test-key"), "{err:?}");
    }

    #[tokio::test]
    async fn get_weather_fetches_every_city() {
        let server = MockServer::start().await;
        for id in ["1", "2"] {
            Mock::given(method("GET"))
                .and(query_param("id", id))
                .respond_with(sample_response())
                .expect(1)
                .mount(&server)
                .await;
        }

        let client = ConfigClient::with_endpoint(&config(vec![1, 2]), &server.uri()).unwrap();
        let stream = client.get_weather();
        tokio::pin!(stream);
        let mut count = 0;
        while let Some(call) = stream.next().await {
            call.await.expect("no timeout").expect("successful call");
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn mask_replaces_key_keeping_first_three_chars() {
        assert_eq!(
            mask("https://x/?appid=abcdef&id=1", "abcdef"),
            "https://x/?appid=abc****&id=1"
        );
    }

    #[test]
    fn mask_short_and_empty_keys() {
        assert_eq!(mask("appid=abc", "abc"), "appid=****");
        assert_eq!(mask("appid=", ""), "appid=");
    }

    #[test]
    fn mask_handles_multibyte_keys() {
        assert_eq!(mask("appid=aé1234", "aé1234"), "appid=aé1****");
    }
}
