//! Data models for the OpenWeatherMap current-weather API response.
//!
//! These mirror the response schema documented at
//! <https://openweathermap.org/current> and were previously provided by the
//! unmaintained `openweathermap_client` crate. They are vendored here so the
//! HTTP/TLS stack can be upgraded to `reqwest` + `rustls` 0.23.

use serde::Deserialize;
use std::fmt;

/// City geo location.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

/// Weather condition description.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Weather {
    pub id: u64,
    pub main: String,
    pub description: String,
    pub icon: String,
}

/// Main readings.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Main {
    pub temp: f64,
    pub feels_like: f64,
    pub temp_min: f64,
    pub temp_max: f64,
    pub pressure: f64,
    #[serde(default)]
    pub sea_level: Option<f64>,
    #[serde(default)]
    pub grnd_level: Option<f64>,
    pub humidity: f64,
}

/// Wind report.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Wind {
    pub speed: f64,
    pub deg: f64,
    #[serde(default)]
    pub gust: Option<f64>,
}

/// Cloudiness readings.
#[derive(Debug, Deserialize)]
pub struct Clouds {
    #[serde(rename = "all")]
    pub cloudiness: f64,
}

/// 1- and 3-hour precipitation amounts, used for both rain and snow.
#[derive(Debug, Deserialize)]
pub struct PrecipVolume {
    #[serde(rename = "1h", default)]
    pub one_hour: Option<f64>,
    #[serde(rename = "3h", default)]
    pub three_hour: Option<f64>,
}

/// Additional information (sunrise/sunset, country).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Sys {
    #[serde(default)]
    pub type_: Option<u64>,
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub message: Option<f64>,
    #[serde(default)]
    pub country: Option<String>,
    pub sunrise: i64,
    pub sunset: i64,
}

/// Main structure for responses from the OpenWeatherMap current-weather API.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CurrentWeather {
    pub coord: Coord,
    pub weather: Vec<Weather>,
    pub base: String,
    pub main: Main,
    pub visibility: u64,
    pub wind: Wind,
    pub clouds: Clouds,
    #[serde(default)]
    pub rain: Option<PrecipVolume>,
    #[serde(default)]
    pub snow: Option<PrecipVolume>,
    pub dt: i64,
    pub sys: Sys,
    pub timezone: i64,
    pub id: u64,
    pub name: String,
    pub cod: u64,
}

/// Used to query the weather in a particular city using its OpenWeatherMap id.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CityId {
    pub id: u32,
}

impl CityId {
    pub fn new(id: u32) -> CityId {
        CityId { id }
    }
}

impl fmt::Display for CityId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
