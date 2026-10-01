use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
    #[default]
    Offline,
    Hangar,
    Loading,
    Battle,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleKind {
    Aircraft,
    Ground,
    Naval,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Vehicle {
    pub technical_name: String,
    pub display_name: String,
    pub kind: VehicleKind,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Telemetry {
    pub speed_ias_kph: Option<f64>,
    pub speed_tas_kph: Option<f64>,
    pub altitude_agl_m: Option<f64>,
    pub altitude_msl_m: Option<f64>,
    pub speed_ground_kph: Option<f64>,
    pub crew_current: Option<u8>,
    pub crew_total: Option<u8>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct BattleId(pub Uuid);

impl BattleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for BattleId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GameSnapshot {
    pub phase: GamePhase,
    pub battle_id: Option<BattleId>,
    pub vehicle: Option<Vehicle>,
    pub telemetry: Telemetry,
    pub map: Option<String>,
    pub mode: Option<String>,
    pub captured_at: DateTime<Utc>,
}

impl Default for GameSnapshot {
    fn default() -> Self {
        Self {
            phase: GamePhase::Offline,
            battle_id: None,
            vehicle: None,
            telemetry: Telemetry::default(),
            map: None,
            mode: None,
            captured_at: Utc::now(),
        }
    }
}

