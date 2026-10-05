use chrono::{TimeZone, Utc};
use wt_presence::{
    domain::{BattleId, GamePhase, GameSnapshot, Telemetry, Vehicle, VehicleKind},
    presence::{PresencePreset, PresenceRenderer},
    session::SessionEngine,
};

fn context() -> (GameSnapshot, wt_presence::session::SessionSummary) {
    let now = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let snapshot = GameSnapshot {
        phase: GamePhase::Battle,
        battle_id: Some(BattleId::new()),
        vehicle: Some(Vehicle {
            technical_name: "j_7d".to_owned(),
            display_name: "J-7D".to_owned(),
            kind: VehicleKind::Aircraft,
        }),
        telemetry: Telemetry {
            speed_ias_kph: Some(912.4),
            altitude_agl_m: Some(31.0),
            ..Telemetry::default()
        },
        mode: Some("Air Simulator".to_owned()),
        captured_at: now,
        ..GameSnapshot::default()
    };
    let mut session = SessionEngine::new(now);
    let summary = session.apply(&snapshot, &[]);
    (snapshot, summary)
}

#[test]
fn renders_configurable_presence_from_normalized_context() {
    let (snapshot, session) = context();
    let preset = PresencePreset {
        id: "test".to_owned(),
        name: "Test".to_owned(),
        details_template: "{{ vehicle.name }} · {{ game.mode }}".to_owned(),
        state_template: "{{ telemetry.ias | round }} km/h · {{ telemetry.agl | round }} m"
            .to_owned(),
        large_image: Some("aircraft".to_owned()),
        small_image: Some("wt".to_owned()),
        show_elapsed: true,
    };

    let activity = PresenceRenderer::new()
        .render(&snapshot, &session, &preset)
        .unwrap();

    assert_eq!(activity.details.as_deref(), Some("J-7D · Air Simulator"));
    assert_eq!(activity.state.as_deref(), Some("912 km/h · 31 m"));
    assert_eq!(activity.large_image.as_deref(), Some("aircraft"));
    assert_eq!(activity.started_at, Some(1_700_000_000));
}

#[test]
fn keeps_session_start_when_presence_status_changes() {
    let session_start = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let mut session = SessionEngine::new(session_start);
    let preset = PresencePreset {
        show_elapsed: true,
        ..PresencePreset::minimal()
    };

    let hangar = GameSnapshot {
        phase: GamePhase::Hangar,
        captured_at: Utc.timestamp_opt(1_700_000_030, 0).unwrap(),
        ..GameSnapshot::default()
    };
    let hangar_summary = session.apply(&hangar, &[]);
    let hangar_activity = PresenceRenderer::new()
        .render(&hangar, &hangar_summary, &preset)
        .unwrap();

    let battle = GameSnapshot {
        phase: GamePhase::Battle,
        battle_id: Some(BattleId::new()),
        vehicle: Some(Vehicle {
            technical_name: "j_7d".to_owned(),
            display_name: "J-7D".to_owned(),
            kind: VehicleKind::Aircraft,
        }),
        captured_at: Utc.timestamp_opt(1_700_000_090, 0).unwrap(),
        ..GameSnapshot::default()
    };
    let battle_summary = session.apply(&battle, &[]);
    let battle_activity = PresenceRenderer::new()
        .render(&battle, &battle_summary, &preset)
        .unwrap();

    assert_eq!(hangar_activity.started_at, Some(1_700_000_000));
    assert_eq!(battle_activity.started_at, Some(1_700_000_000));
}

#[test]
fn unknown_template_variable_returns_an_actionable_error() {
    let (snapshot, session) = context();
    let preset = PresencePreset {
        details_template: "{{ missing.value }}".to_owned(),
        ..PresencePreset::minimal()
    };

    let error = PresenceRenderer::new()
        .render(&snapshot, &session, &preset)
        .unwrap_err();

    assert_eq!(error.kind(), "template");
    assert!(error.to_string().contains("missing"));
}

#[test]
fn rejects_discord_text_over_128_characters() {
    let (snapshot, session) = context();
    let preset = PresencePreset {
        details_template: "x".repeat(129),
        ..PresencePreset::minimal()
    };

    let error = PresenceRenderer::new()
        .render(&snapshot, &session, &preset)
        .unwrap_err();

    assert_eq!(error.kind(), "field_too_long");
    assert!(error.to_string().contains("details"));
}

#[test]
fn minimal_preset_has_a_useful_offline_fallback() {
    let now = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let snapshot = GameSnapshot {
        captured_at: now,
        ..GameSnapshot::default()
    };
    let session = SessionEngine::new(now).summary();

    let activity = PresenceRenderer::new()
        .render(&snapshot, &session, &PresencePreset::minimal())
        .unwrap();

    assert_eq!(activity.details.as_deref(), Some("War Thunder"));
    assert_eq!(activity.state.as_deref(), Some("Offline"));
}

#[test]
fn automatic_artwork_tracks_phase_and_vehicle_domain() {
    let (air_battle, session) = context();
    let hangar = GameSnapshot {
        phase: GamePhase::Hangar,
        ..air_battle.clone()
    };
    let mut ground_battle = air_battle.clone();
    ground_battle.vehicle.as_mut().unwrap().kind = VehicleKind::Ground;
    let mut naval_battle = air_battle.clone();
    naval_battle.vehicle.as_mut().unwrap().kind = VehicleKind::Naval;
    let mut unknown = air_battle.clone();
    unknown.vehicle.as_mut().unwrap().kind = VehicleKind::Unknown;
    let render = |snapshot: &GameSnapshot| {
        PresenceRenderer::new()
            .render(snapshot, &session, &PresencePreset::minimal())
            .unwrap()
    };

    assert_eq!(render(&hangar).large_image.as_deref(), Some("presence-hangar"));
    assert_eq!(
        render(&air_battle).large_image.as_deref(),
        Some("presence-air")
    );
    assert_eq!(
        render(&ground_battle).large_image.as_deref(),
        Some("presence-ground")
    );
    assert_eq!(
        render(&naval_battle).large_image.as_deref(),
        Some("presence-naval")
    );
    assert_eq!(
        render(&unknown).large_image.as_deref(),
        Some("presence-default")
    );
}

#[test]
fn automatic_artwork_defaults_for_loading_offline_and_missing_vehicle() {
    let (battle, session) = context();
    for snapshot in [
        GameSnapshot {
            phase: GamePhase::Loading,
            ..battle.clone()
        },
        GameSnapshot {
            phase: GamePhase::Offline,
            ..battle.clone()
        },
        GameSnapshot {
            vehicle: None,
            ..battle
        },
    ] {
        let activity = PresenceRenderer::new()
            .render(&snapshot, &session, &PresencePreset::minimal())
            .unwrap();

        assert_eq!(activity.large_image.as_deref(), Some("presence-default"));
    }
}

#[test]
fn legacy_default_artwork_uses_canonical_asset() {
    let (snapshot, session) = context();
    let preset = PresencePreset {
        large_image: Some("war_thunder".to_owned()),
        ..PresencePreset::minimal()
    };

    let activity = PresenceRenderer::new()
        .render(&snapshot, &session, &preset)
        .unwrap();

    assert_eq!(activity.large_image.as_deref(), Some("presence-default"));
}

#[test]
fn explicit_canonical_artwork_overrides_automatic_selection() {
    let (snapshot, session) = context();
    for key in [
        "presence-default",
        "presence-air",
        "presence-ground",
        "presence-naval",
        "presence-hangar",
    ] {
        let preset = PresencePreset {
            large_image: Some(key.to_owned()),
            ..PresencePreset::minimal()
        };
        let activity = PresenceRenderer::new()
            .render(&snapshot, &session, &preset)
            .unwrap();

        assert_eq!(activity.large_image.as_deref(), Some(key));
    }
}
