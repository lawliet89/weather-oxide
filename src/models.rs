//! Data models for the OpenWeatherMap current-weather API response.
//!
//! These mirror the response schema documented at
//! <https://openweathermap.org/current> and were previously provided by the
//! unmaintained `openweathermap_client` crate. They are vendored here so the
//! HTTP/TLS stack can be upgraded to `reqwest` + `rustls` 0.23.

use serde::Deserialize;

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
    #[serde(rename = "type", default)]
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
    #[serde(default)]
    pub visibility: Option<u64>,
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

#[cfg(test)]
mod tests {
    use super::CurrentWeather;

    const SAMPLE: &str = r#"{
        "coord": {"lon": 103.85, "lat": 1.29},
        "weather": [{"id": 803, "main": "Clouds", "description": "broken clouds", "icon": "04d"}],
        "base": "stations",
        "main": {"temp": 30.1, "feels_like": 35.2, "temp_min": 29.0, "temp_max": 31.0, "pressure": 1009, "humidity": 70},
        "visibility": 10000,
        "wind": {"speed": 4.1, "deg": 150},
        "clouds": {"all": 75},
        "dt": 1700000000,
        "sys": {"type": 2, "id": 2075535, "country": "SG", "sunrise": 1699999000, "sunset": 1700042000},
        "timezone": 28800,
        "id": 1880252,
        "name": "Singapore",
        "cod": 200
    }"#;

    #[test]
    fn deserializes_sys_type() {
        let weather: CurrentWeather = serde_json::from_str(SAMPLE).unwrap();
        assert_eq!(weather.sys.type_, Some(2));
        assert_eq!(weather.visibility, Some(10000));
    }

    #[test]
    fn missing_visibility_is_none() {
        let mut value: serde_json::Value = serde_json::from_str(SAMPLE).unwrap();
        value.as_object_mut().unwrap().remove("visibility");
        let weather: CurrentWeather = serde_json::from_value(value).unwrap();
        assert_eq!(weather.visibility, None);
    }
}
