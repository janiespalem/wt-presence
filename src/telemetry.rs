use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

use crate::{
    domain::{Telemetry, VehicleKind},
    state::Observation,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PayloadSet {
    pub indicators: Value,
    pub state: Value,
    pub map_info: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NormalizedTelemetry {
    pub observation: Observation,
    pub telemetry: Telemetry,
    pub map: Option<String>,
    pub mode: Option<String>,
}

#[derive(Clone, Debug)]
pub struct WtTelemetryClient {
    base_url: reqwest::Url,
    client: reqwest::Client,
}

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("invalid War Thunder telemetry URL: {0}")]
    InvalidBaseUrl(#[from] url::ParseError),
    #[error("War Thunder telemetry is unreachable: {0}")]
    Unreachable(#[source] reqwest::Error),
    #[error("War Thunder returned an invalid response: {0}")]
    InvalidResponse(#[source] reqwest::Error),
}

impl TelemetryError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::InvalidBaseUrl(_) => "invalid_base_url",
            Self::Unreachable(_) => "unreachable",
            Self::InvalidResponse(_) => "invalid_response",
        }
    }
}

impl WtTelemetryClient {
    pub fn new(base_url: String) -> Result<Self, TelemetryError> {
        let base_url = reqwest::Url::parse(&format!("{}/", base_url.trim_end_matches('/')))?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(750))
            .build()
            .expect("reqwest client configuration is valid");
        Ok(Self { base_url, client })
    }

    pub async fn poll_raw(&self) -> Result<PayloadSet, TelemetryError> {
        let indicators = self.get_json("indicators");
        let state = self.get_json("state");
        let map_info = self.get_json("map_info.json");
        let (indicators, state, map_info) = tokio::try_join!(indicators, state, map_info)?;
        Ok(PayloadSet {
            indicators,
            state,
            map_info,
        })
    }

    async fn get_json(&self, path: &str) -> Result<Value, TelemetryError> {
        let url = self
            .base_url
            .join(path)
            .expect("fixed telemetry endpoint is a valid relative URL");
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(TelemetryError::Unreachable)?;
        response
            .error_for_status()
            .map_err(TelemetryError::InvalidResponse)?
            .json::<Value>()
            .await
            .map_err(TelemetryError::InvalidResponse)
    }
}

pub fn normalize_payloads(payloads: PayloadSet) -> NormalizedTelemetry {
    let army = string_field(&payloads.indicators, "army");
    let vehicle_kind = match army.as_deref() {
        Some("air" | "aircraft" | "dummy_plane") => VehicleKind::Aircraft,
        Some("tank" | "ground" | "dummy_tank") => VehicleKind::Ground,
        Some("ship" | "naval" | "dummy_ship") => VehicleKind::Naval,
        _ => VehicleKind::Unknown,
    };

    let vehicle_name = string_field(&payloads.indicators, "type");
    let map_generation = unsigned_field(&payloads.map_info, "map_generation");
    let map_valid =
        boolean_field(&payloads.map_info, "valid").unwrap_or_else(|| map_generation.is_some());

    let telemetry = Telemetry {
        speed_ias_kph: number_field(&payloads.state, "IAS, km/h"),
        speed_tas_kph: number_field(&payloads.state, "TAS, km/h"),
        altitude_agl_m: number_field(&payloads.indicators, "radio_altitude"),
        altitude_msl_m: number_field(&payloads.state, "H, m"),
        speed_ground_kph: number_field(&payloads.indicators, "speed"),
        crew_current: unsigned_field(&payloads.indicators, "crew_current")
            .and_then(|value| u8::try_from(value).ok()),
        crew_total: unsigned_field(&payloads.indicators, "crew_total")
            .and_then(|value| u8::try_from(value).ok()),
    };

    NormalizedTelemetry {
        observation: Observation {
            reachable: true,
            map_valid,
            vehicle_name,
            vehicle_kind,
            map_generation,
        },
        telemetry,
        map: string_field(&payloads.map_info, "map_name"),
        mode: string_field(&payloads.map_info, "game_mode")
            .or_else(|| string_field(&payloads.map_info, "mode")),
    }
}

fn boolean_field(value: &Value, key: &str) -> Option<bool> {
    value.get(key)?.as_bool()
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(str::to_owned)
}

fn number_field(value: &Value, key: &str) -> Option<f64> {
    let value = value.get(key)?;
    value
        .as_f64()
        .or_else(|| value.as_str()?.parse::<f64>().ok())
        .filter(|number| number.is_finite())
}

fn unsigned_field(value: &Value, key: &str) -> Option<u64> {
    let value = value.get(key)?;
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse::<u64>().ok())
}
