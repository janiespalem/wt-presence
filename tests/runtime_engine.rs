use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use tempfile::tempdir;
use wt_presence::{
    api::ApiState,
    config::{AppSettings, SettingsStore},
    domain::{Telemetry, VehicleKind},
    presence::DiscordActivity,
    runtime::{PresenceSink, RuntimeEngine, TelemetrySource},
    state::Observation,
    storage::SessionRepository,
    telemetry::NormalizedTelemetry,
};

struct FakeSource(Result<NormalizedTelemetry, String>);

#[async_trait]
impl TelemetrySource for FakeSource {
    async fn poll(&self) -> Result<NormalizedTelemetry, String> {
        self.0.clone()
    }
}

#[derive(Clone, Default)]
struct FakeSink {
    events: Arc<Mutex<Vec<String>>>,
}

impl PresenceSink for FakeSink {
    fn publish(&mut self, activity: &DiscordActivity) -> Result<(), String> {
        self.events.lock().unwrap().push(format!(
            "publish:{}",
            activity.details.as_deref().unwrap_or_default()
        ));
        Ok(())
    }

    fn clear(&mut self) -> Result<(), String> {
        self.events.lock().unwrap().push("clear".to_owned());
        Ok(())
    }
}

fn api_state(directory: &tempfile::TempDir) -> ApiState {
    ApiState::new(
        AppSettings::default(),
        SettingsStore::new(directory.path().join("settings.json")),
        SessionRepository::open_in_memory().unwrap(),
        "token",
        "http://127.0.0.1:32147",
    )
}

#[tokio::test]
async fn one_tick_publishes_presence_and_persists_the_session() {
    let directory = tempdir().unwrap();
    let api = api_state(&directory);
    let sink = FakeSink::default();
    let events = sink.events.clone();
    let source = FakeSource(Ok(NormalizedTelemetry {
        observation: Observation {
            reachable: true,
            map_valid: true,
            vehicle_name: Some("j_7d".to_owned()),
            vehicle_kind: VehicleKind::Aircraft,
            map_generation: Some(9),
        },
        telemetry: Telemetry {
            speed_ias_kph: Some(930.0),
            ..Telemetry::default()
        },
        map: Some("Sinai".to_owned()),
        mode: Some("Air Simulator".to_owned()),
    }));
    let now = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let mut runtime = RuntimeEngine::new(source, sink, api.clone(), now);

    runtime.tick(now).await;

    assert_eq!(events.lock().unwrap().as_slice(), ["publish:j 7d"]);
    assert!(api.status().await.telemetry_connected);
    assert!(api.status().await.discord_connected);
    assert_eq!(api.snapshot().await.map.as_deref(), Some("Sinai"));
    assert_eq!(api.recent_sessions(10).unwrap().len(), 1);
}

#[tokio::test]
async fn an_unreachable_game_clears_presence_without_stopping_the_runtime() {
    let directory = tempdir().unwrap();
    let api = api_state(&directory);
    let sink = FakeSink::default();
    let events = sink.events.clone();
    let now = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let mut runtime = RuntimeEngine::new(
        FakeSource(Err("War Thunder is unreachable".to_owned())),
        sink,
        api.clone(),
        now,
    );

    runtime.tick(now).await;

    assert_eq!(events.lock().unwrap().as_slice(), ["clear"]);
    let status = api.status().await;
    assert!(!status.telemetry_connected);
    assert!(!status.discord_connected);
    assert!(status.last_error.unwrap().contains("unreachable"));
}
