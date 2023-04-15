use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use openweathermap_client::models::CurrentWeather;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct LegacyRecord {
    #[serde(rename = "")]
    index: u8,

    /// City
    pub city: String,

    // Weather
    /// Description
    description: String,
    /// Icon
    icon: String,
    /// Main_Weather
    #[serde(rename = "Main_Weather")]
    main_weather: String,

    /// ID
    #[serde(rename = "ID")]
    city_id: u64,

    /// Visibility[m]
    #[serde(rename = "Visibility[m]")]
    visibility: u64,
    /// Humidity[%]
    #[serde(rename = "Humidity[%]")]
    humidity: f64,
    /// Pressure[hPa]
    #[serde(rename = "Pressure[hPa]")]
    pressure: f64,

    /// Time
    time: String,
    /// UNIX_UTC
    #[serde(rename = "UNIX_UTC")]
    unix_utc: i64,

    /// Rain[3h][mm]
    #[serde(rename = "Rain[3h][mm]")]
    rain_3h: f64,
    /// Rain[1h][mm]
    #[serde(rename = "Rain[1h][mm]")]
    rain_1h: f64,

    /// Snow[3h][mm]
    #[serde(rename = "Snow[3h][mm]")]
    snow_3h: f64,
    /// Snow[3h][mm]
    #[serde(rename = "Snow[1h][mm]")]
    snow_1h: f64,

    /// Min_temp
    #[serde(rename = "Min_temp")]
    temp_min: f64,
    /// Max_temp
    #[serde(rename = "Max_temp")]
    temp_max: f64,
    /// Temp
    temp: f64,

    // Sys
    /// Country
    country: String,
    // Sunrise
    sunrise: i64,
    // Sunset
    sunset: i64,

    /// Clouds[%]
    #[serde(rename = "Clouds[%]")]
    clouds: f64,
    /// Wind_direction
    #[serde(rename = "Wind_direction")]
    wind_direction: f64,
    // Wind_speed[m/s]
    #[serde(rename = "Wind_speed[m/s]")]
    wind_speed: f64,

    // Latitude
    latitude: f64,
    // Longitude
    longtitude: f64,

    #[serde(skip_serializing)]
    date_time: DateTime<Utc>,
}

impl LegacyRecord {
    pub fn path<P: AsRef<Path>>(&self, base: P, extension: &str) -> PathBuf {
        let mut path = PathBuf::new();
        path.push(base.as_ref());
        path.push(&self.city);

        path.push(format!("{}.{}", self.date_time.year(), extension));

        path
    }

    pub fn append<P: AsRef<Path>>(
        &self,
        base: P,
        extension: &str,
        delimiter: char,
    ) -> Result<(), anyhow::Error> {
        let path = self.path(base, extension);
        let file_exists = path.exists();

        let file = File::options()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| "cannot open file to append")?;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(delimiter as u8)
            .has_headers(!file_exists)
            .from_writer(file);
        writer
            .serialize(self)
            .with_context(|| "error writing record")?;
        Ok(())
    }
}

impl From<CurrentWeather> for LegacyRecord {
    fn from(weather: CurrentWeather) -> Self {
        let city = weather.name;

        let reading = weather.weather.get(0).expect("to exist");

        let description = reading.description.clone();
        let icon = reading.icon.clone();
        let main_weather = reading.main.clone();

        let city_id = weather.id;

        let visibility = weather.visibility;
        let humidity = weather.main.humidity;
        let pressure = weather.main.pressure;

        let date_time = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(weather.dt, 0).expect("to exist"),
            Utc,
        );
        let time = date_time.format("%Y-%m-%d %H:%M:%S");

        let (rain_3h, rain_1h) = match weather.rain {
            None => (0.0, 0.0),
            Some(precip) => (
                precip.three_hour.unwrap_or_default(),
                precip.one_hour.unwrap_or_default(),
            ),
        };

        let (snow_3h, snow_1h) = match weather.snow {
            None => (0.0, 0.0),
            Some(precip) => (
                precip.three_hour.unwrap_or_default(),
                precip.one_hour.unwrap_or_default(),
            ),
        };

        let temp_min = weather.main.temp_min;
        let temp_max = weather.main.temp_max;
        let temp = weather.main.temp;

        let country = weather.sys.country.unwrap_or_default();
        let sunrise = weather.sys.sunrise;
        let sunset = weather.sys.sunset;

        let clouds = weather.clouds.cloudiness;
        let wind_direction = weather.wind.deg;
        let wind_speed = weather.wind.speed;

        let latitude = weather.coord.lat;
        let longtitude = weather.coord.lon;

        Self {
            index: 0,
            city,
            description,
            icon,
            main_weather,
            city_id,
            visibility,
            humidity,
            pressure,
            time: time.to_string(),
            unix_utc: weather.dt,
            rain_3h,
            rain_1h,
            snow_3h,
            snow_1h,
            temp_min,
            temp_max,
            temp,
            country,
            sunrise,
            sunset,
            clouds,
            wind_direction,
            wind_speed,
            latitude,
            longtitude,

            date_time,
        }
    }
}
