use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use tempfile::tempdir;
use wt_presence::{
    api::ApiState,
    config::{AppSettings, SettingsStore},
    domain::{Telemetry, VehicleKind},
    presence::DiscordActivity,
    runtime::{PresencePublisher, PresenceSink, PublishOutcome, RuntimeEngine, TelemetrySource},
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

struct SequencedSource(Mutex<VecDeque<NormalizedTelemetry>>);

#[async_trait]
impl TelemetrySource for SequencedSource {
    async fn poll(&self) -> Result<NormalizedTelemetry, String> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .pop_front()
            .expect("telemetry fixture exhausted"))
    }
}

struct SequencedResultSource(Mutex<VecDeque<Result<NormalizedTelemetry, String>>>);

#[async_trait]
impl TelemetrySource for SequencedResultSource {
    async fn poll(&self) -> Result<NormalizedTelemetry, String> {
        self.0
            .lock()
            .unwrap()
            .pop_front()
            .expect("telemetry fixture exhausted")
    }
}

#[derive(Debug, PartialEq)]
enum SinkEvent {
    Publish(DiscordActivity),
    Clear,
}

#[derive(Default)]
struct FakeSink {
    events: Arc<Mutex<Vec<SinkEvent>>>,
    publish_results: VecDeque<Result<(), String>>,
    clear_error: Option<String>,
}

impl PresenceSink for FakeSink {
    fn publish(&mut self, activity: &DiscordActivity) -> Result<(), String> {
        self.events
            .lock()
            .unwrap()
            .push(SinkEvent::Publish(activity.clone()));
        self.publish_results.pop_front().unwrap_or(Ok(()))
    }

    fn clear(&mut self) -> Result<(), String> {
        self.events.lock().unwrap().push(SinkEvent::Clear);
        self.clear_error.take().map_or(Ok(()), Err)
    }
}

fn activity() -> DiscordActivity {
    DiscordActivity {
        details: Some("J-7D".to_owned()),
        state: Some("In battle".to_owned()),
        large_image: Some("presence-air".to_owned()),
        small_image: Some("squadron".to_owned()),
        started_at: Some(1_700_000_000),
    }
}

fn at(seconds: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(1_700_000_000 + seconds, 0).unwrap()
}

#[test]
fn suppresses_identical_activities_until_heartbeat_but_publishes_changes() {
    let sink = FakeSink::default();
    let events = sink.events.clone();
    let mut publisher = PresencePublisher::new(sink, Duration::from_secs(15));
    let initial = activity();
    let changed = DiscordActivity {
        state: Some("Returning to hangar".to_owned()),
        ..initial.clone()
    };

    assert_eq!(
        publisher.publish(&initial, at(0)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        publisher.publish(&initial, at(1)),
        Ok(PublishOutcome::Skipped)
    );
    assert_eq!(
        publisher.publish(&changed, at(2)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        publisher.publish(&changed, at(16)),
        Ok(PublishOutcome::Skipped)
    );
    assert_eq!(
        publisher.publish(&changed, at(17)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(initial),
            SinkEvent::Publish(changed.clone()),
            SinkEvent::Publish(changed),
        ]
    );
}

#[test]
fn clock_rollback_refreshes_identical_activity_and_restarts_heartbeat() {
    let sink = FakeSink::default();
    let events = sink.events.clone();
    let mut publisher = PresencePublisher::new(sink, Duration::from_secs(15));
    let activity = activity();

    assert_eq!(
        publisher.publish(&activity, at(30)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        publisher.publish(&activity, at(0)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        publisher.publish(&activity, at(14)),
        Ok(PublishOutcome::Skipped)
    );
    assert_eq!(
        publisher.publish(&activity, at(15)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(activity.clone()),
            SinkEvent::Publish(activity.clone()),
            SinkEvent::Publish(activity),
        ]
    );
}

#[test]
fn every_activity_field_change_publishes_immediately() {
    let initial = activity();
    let changes = [
        DiscordActivity {
            details: Some("F-16C".to_owned()),
            ..initial.clone()
        },
        DiscordActivity {
            state: Some("In hangar".to_owned()),
            ..initial.clone()
        },
        DiscordActivity {
            large_image: Some("presence-ground".to_owned()),
            ..initial.clone()
        },
        DiscordActivity {
            small_image: None,
            ..initial.clone()
        },
        DiscordActivity {
            started_at: Some(1_700_000_001),
            ..initial.clone()
        },
    ];

    for changed in changes {
        let sink = FakeSink::default();
        let events = sink.events.clone();
        let mut publisher = PresencePublisher::new(sink, Duration::from_secs(15));
        assert_eq!(
            publisher.publish(&initial, at(0)),
            Ok(PublishOutcome::Published)
        );

        assert_eq!(
            publisher.publish(&changed, at(1)),
            Ok(PublishOutcome::Published)
        );
        assert_eq!(
            events.lock().unwrap().as_slice(),
            [
                SinkEvent::Publish(initial.clone()),
                SinkEvent::Publish(changed),
            ]
        );
    }
}

#[test]
fn clear_allows_the_same_activity_after_reconnect() {
    let sink = FakeSink::default();
    let events = sink.events.clone();
    let mut publisher = PresencePublisher::new(sink, Duration::from_secs(15));
    let activity = activity();

    assert_eq!(
        publisher.publish(&activity, at(0)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(publisher.clear(), Ok(()));
    assert_eq!(
        publisher.publish(&activity, at(1)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(activity.clone()),
            SinkEvent::Clear,
            SinkEvent::Publish(activity),
        ]
    );
}

#[test]
fn failed_clear_allows_the_same_activity_after_reconnect() {
    let sink = FakeSink {
        clear_error: Some("Discord disconnected".to_owned()),
        ..FakeSink::default()
    };
    let events = sink.events.clone();
    let mut publisher = PresencePublisher::new(sink, Duration::from_secs(15));
    let activity = activity();

    assert_eq!(
        publisher.publish(&activity, at(0)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(publisher.clear(), Err("Discord disconnected".to_owned()));
    assert_eq!(
        publisher.publish(&activity, at(1)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(activity.clone()),
            SinkEvent::Clear,
            SinkEvent::Publish(activity),
        ]
    );
}

#[test]
fn failed_publish_is_retried_without_waiting_for_heartbeat() {
    let sink = FakeSink {
        publish_results: VecDeque::from([Err("Discord is not running".to_owned())]),
        ..FakeSink::default()
    };
    let events = sink.events.clone();
    let mut publisher = PresencePublisher::new(sink, Duration::from_secs(15));
    let activity = activity();

    assert_eq!(
        publisher.publish(&activity, at(0)),
        Err("Discord is not running".to_owned())
    );
    assert_eq!(
        publisher.publish(&activity, at(1)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(activity.clone()),
            SinkEvent::Publish(activity),
        ]
    );
}

#[test]
fn failed_changed_publish_is_retried_without_waiting_for_heartbeat() {
    let sink = FakeSink {
        publish_results: VecDeque::from([Ok(()), Err("Discord disconnected".to_owned())]),
        ..FakeSink::default()
    };
    let events = sink.events.clone();
    let mut publisher = PresencePublisher::new(sink, Duration::from_secs(15));
    let initial = activity();
    let changed = DiscordActivity {
        state: Some("In hangar".to_owned()),
        ..initial.clone()
    };

    assert_eq!(
        publisher.publish(&initial, at(0)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        publisher.publish(&changed, at(1)),
        Err("Discord disconnected".to_owned())
    );
    assert_eq!(
        publisher.publish(&changed, at(2)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(initial),
            SinkEvent::Publish(changed.clone()),
            SinkEvent::Publish(changed),
        ]
    );
}

#[test]
fn failed_changed_publish_retries_previously_cached_activity() {
    let sink = FakeSink {
        publish_results: VecDeque::from([Ok(()), Err("Discord disconnected".to_owned())]),
        ..FakeSink::default()
    };
    let events = sink.events.clone();
    let mut publisher = PresencePublisher::new(sink, Duration::from_secs(15));
    let initial = activity();
    let changed = DiscordActivity {
        state: Some("In hangar".to_owned()),
        ..initial.clone()
    };

    assert_eq!(
        publisher.publish(&initial, at(0)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        publisher.publish(&changed, at(1)),
        Err("Discord disconnected".to_owned())
    );
    assert_eq!(
        publisher.publish(&initial, at(2)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        publisher.publish(&initial, at(3)),
        Ok(PublishOutcome::Skipped)
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(initial.clone()),
            SinkEvent::Publish(changed),
            SinkEvent::Publish(initial),
        ]
    );
}

#[test]
fn failed_heartbeat_is_retried_without_waiting_for_another_interval() {
    let sink = FakeSink {
        publish_results: VecDeque::from([Ok(()), Err("Discord disconnected".to_owned())]),
        ..FakeSink::default()
    };
    let events = sink.events.clone();
    let mut publisher = PresencePublisher::new(sink, Duration::from_secs(15));
    let activity = activity();

    assert_eq!(
        publisher.publish(&activity, at(0)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        publisher.publish(&activity, at(15)),
        Err("Discord disconnected".to_owned())
    );
    assert_eq!(
        publisher.publish(&activity, at(16)),
        Ok(PublishOutcome::Published)
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(activity.clone()),
            SinkEvent::Publish(activity.clone()),
            SinkEvent::Publish(activity),
        ]
    );
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

    assert_eq!(
        events.lock().unwrap().as_slice(),
        [SinkEvent::Publish(DiscordActivity {
            details: Some("j 7d".to_owned()),
            state: Some("In battle".to_owned()),
            large_image: Some("presence-air".to_owned()),
            small_image: None,
            started_at: None,
        })]
    );
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

    assert_eq!(events.lock().unwrap().as_slice(), [SinkEvent::Clear]);
    let status = api.status().await;
    assert!(!status.telemetry_connected);
    assert!(!status.discord_connected);
    assert!(status.last_error.unwrap().contains("unreachable"));
}

#[tokio::test]
async fn runtime_health_keeps_start_and_last_success_times_across_disconnects() {
    let directory = tempdir().unwrap();
    let api = api_state(&directory);
    let source = SequencedResultSource(Mutex::new(VecDeque::from([
        Ok(NormalizedTelemetry {
            observation: Observation {
                reachable: true,
                map_valid: false,
                vehicle_name: None,
                vehicle_kind: VehicleKind::Unknown,
                map_generation: None,
            },
            telemetry: Telemetry::default(),
            map: None,
            mode: None,
        }),
        Err("War Thunder is unreachable".to_owned()),
    ])));
    let started_at = at(-30);
    let mut runtime = RuntimeEngine::new(source, FakeSink::default(), api.clone(), started_at);

    runtime.tick(at(0)).await;
    let connected = api.status().await;
    assert_eq!(connected.started_at, started_at);
    assert_eq!(connected.last_telemetry_at, Some(at(0)));
    assert_eq!(connected.last_discord_at, Some(at(0)));

    runtime.tick(at(5)).await;
    let disconnected = api.status().await;
    assert_eq!(disconnected.started_at, started_at);
    assert_eq!(disconnected.last_telemetry_at, Some(at(0)));
    assert_eq!(disconnected.last_discord_at, Some(at(0)));
    assert!(!disconnected.telemetry_connected);
    assert!(!disconnected.discord_connected);
}

#[tokio::test]
async fn runtime_suppresses_identical_presence_until_the_fifteen_second_heartbeat() {
    let directory = tempdir().unwrap();
    let api = api_state(&directory);
    let sink = FakeSink::default();
    let events = sink.events.clone();
    let source = FakeSource(Ok(NormalizedTelemetry {
        observation: Observation {
            reachable: true,
            map_valid: false,
            vehicle_name: None,
            vehicle_kind: VehicleKind::Unknown,
            map_generation: None,
        },
        telemetry: Telemetry::default(),
        map: None,
        mode: None,
    }));
    let mut runtime = RuntimeEngine::new(source, sink, api.clone(), at(0));

    runtime.tick(at(0)).await;
    runtime.tick(at(1)).await;
    runtime.tick(at(14)).await;
    runtime.tick(at(15)).await;

    let expected = DiscordActivity {
        details: Some("War Thunder".to_owned()),
        state: Some("In hangar".to_owned()),
        large_image: Some("presence-hangar".to_owned()),
        small_image: None,
        started_at: None,
    };
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(expected.clone()),
            SinkEvent::Publish(expected),
        ]
    );
    assert!(api.status().await.discord_connected);
    assert_eq!(api.snapshot().await.captured_at, at(15));
}

#[tokio::test]
async fn runtime_reconnects_when_discord_starts_after_the_first_tick() {
    let directory = tempdir().unwrap();
    let api = api_state(&directory);
    let sink = FakeSink {
        publish_results: VecDeque::from([Err("Discord is not running".to_owned())]),
        ..FakeSink::default()
    };
    let events = sink.events.clone();
    let source = FakeSource(Ok(NormalizedTelemetry {
        observation: Observation {
            reachable: true,
            map_valid: false,
            vehicle_name: None,
            vehicle_kind: VehicleKind::Unknown,
            map_generation: None,
        },
        telemetry: Telemetry::default(),
        map: None,
        mode: None,
    }));
    let mut runtime = RuntimeEngine::new(source, sink, api.clone(), at(0));

    runtime.tick(at(0)).await;
    let disconnected = api.status().await;
    assert!(!disconnected.discord_connected);
    assert_eq!(
        disconnected.last_error.as_deref(),
        Some("Discord is not running")
    );

    runtime.tick(at(1)).await;

    let expected = DiscordActivity {
        details: Some("War Thunder".to_owned()),
        state: Some("In hangar".to_owned()),
        large_image: Some("presence-hangar".to_owned()),
        small_image: None,
        started_at: None,
    };
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(expected.clone()),
            SinkEvent::Publish(expected),
        ]
    );
    let connected = api.status().await;
    assert!(connected.discord_connected);
    assert!(connected.last_error.is_none());
}

#[tokio::test]
async fn runtime_keeps_disconnected_status_when_retrying_previously_cached_activity() {
    let directory = tempdir().unwrap();
    let api = api_state(&directory);
    let sink = FakeSink {
        publish_results: VecDeque::from([
            Ok(()),
            Err("Discord disconnected".to_owned()),
            Err("Discord still disconnected".to_owned()),
            Ok(()),
        ]),
        ..FakeSink::default()
    };
    let events = sink.events.clone();
    let initial = NormalizedTelemetry {
        observation: Observation {
            reachable: true,
            map_valid: true,
            vehicle_name: Some("j_7d".to_owned()),
            vehicle_kind: VehicleKind::Aircraft,
            map_generation: Some(9),
        },
        telemetry: Telemetry::default(),
        map: Some("Sinai".to_owned()),
        mode: Some("Air Simulator".to_owned()),
    };
    let mut changed = initial.clone();
    changed.observation.vehicle_name = Some("f_16c".to_owned());
    let source = SequencedSource(Mutex::new(VecDeque::from([
        initial.clone(),
        changed,
        initial.clone(),
        initial.clone(),
        initial,
    ])));
    let mut runtime = RuntimeEngine::new(source, sink, api.clone(), at(0));

    runtime.tick(at(0)).await;
    assert!(api.status().await.discord_connected);
    assert!(api.status().await.last_error.is_none());

    runtime.tick(at(1)).await;
    let disconnected = api.status().await;
    assert!(disconnected.telemetry_connected);
    assert!(!disconnected.discord_connected);
    assert_eq!(
        disconnected.last_error.as_deref(),
        Some("Discord disconnected")
    );

    runtime.tick(at(2)).await;
    let still_disconnected = api.status().await;
    assert!(still_disconnected.telemetry_connected);
    assert!(!still_disconnected.discord_connected);
    assert_eq!(
        still_disconnected.last_error.as_deref(),
        Some("Discord still disconnected")
    );

    runtime.tick(at(3)).await;
    let recovered = api.status().await;
    assert!(recovered.discord_connected);
    assert!(recovered.last_error.is_none());

    runtime.tick(at(4)).await;
    assert!(api.status().await.discord_connected);
    assert!(api.status().await.last_error.is_none());

    let expected_initial = DiscordActivity {
        details: Some("j 7d".to_owned()),
        state: Some("In battle".to_owned()),
        large_image: Some("presence-air".to_owned()),
        small_image: None,
        started_at: None,
    };
    let expected_changed = DiscordActivity {
        details: Some("f 16c".to_owned()),
        state: Some("In battle".to_owned()),
        large_image: Some("presence-air".to_owned()),
        small_image: None,
        started_at: None,
    };
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            SinkEvent::Publish(expected_initial.clone()),
            SinkEvent::Publish(expected_changed),
            SinkEvent::Publish(expected_initial.clone()),
            SinkEvent::Publish(expected_initial),
        ]
    );
}
