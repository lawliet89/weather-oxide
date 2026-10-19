use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{DateTime, Datelike, Utc};
use serde::Serialize;

use crate::models::CurrentWeather;

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
    visibility: Option<u64>,
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
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("cannot create directory {}", parent.display()))?;
        }
        let file = File::options()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| "cannot open file to append")?;
        // Check the size rather than existence so an empty file left behind by
        // an earlier failed run still gets a header.
        let is_empty = file
            .metadata()
            .with_context(|| "cannot read file metadata")?
            .len()
            == 0;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(delimiter as u8)
            .has_headers(is_empty)
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

        let reading = weather.weather.first().expect("to exist");

        let description = reading.description.clone();
        let icon = reading.icon.clone();
        let main_weather = reading.main.clone();

        let city_id = weather.id;

        let visibility = weather.visibility;
        let humidity = weather.main.humidity;
        let pressure = weather.main.pressure;

        let date_time = DateTime::<Utc>::from_timestamp(weather.dt, 0).expect("to exist");
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::LegacyRecord;
    use crate::models::{tests::SAMPLE, CurrentWeather};

    const HEADER: &str = ",City,Description,Icon,Main_Weather,ID,Visibility[m],Humidity[%],\
        Pressure[hPa],Time,UNIX_UTC,Rain[3h][mm],Rain[1h][mm],Snow[3h][mm],Snow[1h][mm],\
        Min_temp,Max_temp,Temp,Country,Sunrise,Sunset,Clouds[%],Wind_direction,\
        Wind_speed[m/s],Latitude,Longtitude";

    fn sample_value() -> serde_json::Value {
        serde_json::from_str(SAMPLE).unwrap()
    }

    fn record_from(value: serde_json::Value) -> LegacyRecord {
        let weather: CurrentWeather = serde_json::from_value(value).unwrap();
        LegacyRecord::from(weather)
    }

    /// A fresh, empty directory under the system temp dir, removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("weather-oxide-{}-{}", std::process::id(), name));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn converts_weather_to_record() {
        let record = record_from(sample_value());
        assert_eq!(record.index, 0);
        assert_eq!(record.city, "Singapore");
        assert_eq!(record.description, "broken clouds");
        assert_eq!(record.icon, "04d");
        assert_eq!(record.main_weather, "Clouds");
        assert_eq!(record.city_id, 1880252);
        assert_eq!(record.visibility, Some(10000));
        assert_eq!(record.humidity, 70.0);
        assert_eq!(record.pressure, 1009.0);
        assert_eq!(record.time, "2023-11-14 22:13:20");
        assert_eq!(record.unix_utc, 1700000000);
        assert_eq!(record.temp_min, 29.0);
        assert_eq!(record.temp_max, 31.0);
        assert_eq!(record.temp, 30.1);
        assert_eq!(record.country, "SG");
        assert_eq!(record.sunrise, 1699999000);
        assert_eq!(record.sunset, 1700042000);
        assert_eq!(record.clouds, 75.0);
        assert_eq!(record.wind_direction, 150.0);
        assert_eq!(record.wind_speed, 4.1);
        assert_eq!(record.latitude, 1.29);
        assert_eq!(record.longtitude, 103.85);
    }

    #[test]
    fn missing_precipitation_defaults_to_zero() {
        let record = record_from(sample_value());
        assert_eq!(record.rain_1h, 0.0);
        assert_eq!(record.rain_3h, 0.0);
        assert_eq!(record.snow_1h, 0.0);
        assert_eq!(record.snow_3h, 0.0);
    }

    #[test]
    fn partial_precipitation_fills_missing_with_zero() {
        let mut value = sample_value();
        value["rain"] = serde_json::json!({"1h": 0.4});
        value["snow"] = serde_json::json!({"3h": 1.5});
        let record = record_from(value);
        assert_eq!(record.rain_1h, 0.4);
        assert_eq!(record.rain_3h, 0.0);
        assert_eq!(record.snow_1h, 0.0);
        assert_eq!(record.snow_3h, 1.5);
    }

    #[test]
    fn uses_first_weather_reading() {
        let mut value = sample_value();
        value["weather"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"id": 500, "main": "Rain", "description": "light rain", "icon": "10d"}));
        let record = record_from(value);
        assert_eq!(record.main_weather, "Clouds");
    }

    #[test]
    fn missing_country_becomes_empty() {
        let mut value = sample_value();
        value["sys"].as_object_mut().unwrap().remove("country");
        assert_eq!(record_from(value).country, "");
    }

    #[test]
    fn path_is_base_city_year_extension() {
        let record = record_from(sample_value());
        assert_eq!(
            record.path("/data", "csv"),
            PathBuf::from("/data/Singapore/2023.csv")
        );
        assert_eq!(
            record.path("out", "tsv"),
            PathBuf::from("out/Singapore/2023.tsv")
        );
    }

    #[test]
    fn append_writes_header_only_once() {
        let dir = TempDir::new("header-once");
        let record = record_from(sample_value());

        record.append(dir.path(), "csv", ',').unwrap();
        record.append(dir.path(), "csv", ',').unwrap();

        let contents = std::fs::read_to_string(record.path(dir.path(), "csv")).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 3, "{contents}");
        assert_eq!(lines[0], HEADER);
        assert_eq!(lines[1], lines[2]);
        assert!(lines[1].starts_with("0,Singapore,broken clouds,04d,Clouds,1880252,10000,"));
    }

    #[test]
    fn append_writes_empty_field_for_missing_visibility() {
        let dir = TempDir::new("no-visibility");
        let mut value = sample_value();
        value.as_object_mut().unwrap().remove("visibility");
        let record = record_from(value);

        record.append(dir.path(), "csv", ',').unwrap();

        let contents = std::fs::read_to_string(record.path(dir.path(), "csv")).unwrap();
        let row = contents.lines().nth(1).unwrap();
        assert!(
            row.starts_with("0,Singapore,broken clouds,04d,Clouds,1880252,,"),
            "{row}"
        );
    }

    #[test]
    fn append_uses_configured_delimiter() {
        let dir = TempDir::new("delimiter");
        let record = record_from(sample_value());

        record.append(dir.path(), "tsv", '\t').unwrap();

        let contents = std::fs::read_to_string(record.path(dir.path(), "tsv")).unwrap();
        let header = contents.lines().next().unwrap();
        assert_eq!(header, HEADER.replace(',', "\t"));
    }

    #[test]
    fn append_writes_header_to_existing_empty_file() {
        let dir = TempDir::new("empty-file");
        let record = record_from(sample_value());
        let path = record.path(dir.path(), "csv");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::File::create(&path).unwrap();

        record.append(dir.path(), "csv", ',').unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().next().unwrap(), HEADER);
        assert_eq!(contents.lines().count(), 2);
    }

    #[test]
    fn append_creates_missing_directories() {
        let dir = TempDir::new("missing-city-dir");
        let base = dir.path().join("nested").join("output");
        let record = record_from(sample_value());

        record.append(&base, "csv", ',').unwrap();

        let contents = std::fs::read_to_string(record.path(&base, "csv")).unwrap();
        assert_eq!(contents.lines().next().unwrap(), HEADER);
        assert_eq!(contents.lines().count(), 2);
    }
}
