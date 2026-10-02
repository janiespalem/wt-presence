use chrono::{TimeZone, Utc};
use wt_presence::{
    domain::GameSnapshot,
    presence::PresencePreset,
    preview::{PreviewScenario, render_preview},
    session::SessionEngine,
};

fn live_context() -> (GameSnapshot, wt_presence::session::SessionSummary) {
    let now = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let snapshot = GameSnapshot {
        captured_at: now,
        ..GameSnapshot::default()
    };
    let session = SessionEngine::new(now).summary();
    (snapshot, session)
}

fn diagnostic_preset() -> PresencePreset {
    PresencePreset {
        id: "diagnostic".to_owned(),
        name: "Diagnostic".to_owned(),
        details_template: "{{ vehicle.name }} · {{ game.mode }}".to_owned(),
        state_template: "{{ telemetry.ias | round }} km/h · {{ telemetry.agl | round }} m · {{ session.kills }} kills".to_owned(),
        large_image: None,
        small_image: None,
        show_elapsed: false,
    }
}

#[test]
fn air_scenario_renders_without_live_telemetry() {
    let (snapshot, session) = live_context();

    let activity = render_preview(
        &snapshot,
        &session,
        &diagnostic_preset(),
        PreviewScenario::Air,
    )
    .unwrap();

    assert_eq!(activity.details.as_deref(), Some("J-7D · Air Realistic"));
    assert_eq!(
        activity.state.as_deref(),
        Some("1080 km/h · 75 m · 6 kills")
    );
}

#[test]
fn ground_scenario_exposes_ground_specific_signals() {
    let (snapshot, session) = live_context();
    let preset = PresencePreset {
        state_template: "{{ telemetry.ground_speed | round }} km/h · crew {{ telemetry.crew_current }}/{{ telemetry.crew_total }} · {{ game.map }}".to_owned(),
        ..diagnostic_preset()
    };

    let activity = render_preview(
        &snapshot,
        &session,
        &preset,
        PreviewScenario::Ground,
    )
    .unwrap();

    assert_eq!(
        activity.details.as_deref(),
        Some("Al-Khalid-I · Ground Realistic")
    );
    assert_eq!(
        activity.state.as_deref(),
        Some("48 km/h · crew 3/3 · Sinai")
    );
}
