use serde::{Deserialize, Serialize};

use crate::{
    domain::{BattleId, GamePhase, GameSnapshot, Telemetry, Vehicle, VehicleKind},
    presence::{DiscordActivity, PresenceError, PresencePreset, PresenceRenderer},
    session::SessionSummary,
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewScenario {
    #[default]
    Live,
    Hangar,
    Air,
    Ground,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PreviewRequest {
    pub preset: PresencePreset,
    #[serde(default)]
    pub scenario: PreviewScenario,
}

pub fn render_preview(
    live_snapshot: &GameSnapshot,
    live_session: &SessionSummary,
    preset: &PresencePreset,
    scenario: PreviewScenario,
) -> Result<DiscordActivity, PresenceError> {
    let (snapshot, session) = preview_context(live_snapshot, live_session, scenario);
    PresenceRenderer::new().render(&snapshot, &session, preset)
}

fn preview_context(
    live_snapshot: &GameSnapshot,
    live_session: &SessionSummary,
    scenario: PreviewScenario,
) -> (GameSnapshot, SessionSummary) {
    if scenario == PreviewScenario::Live {
        return (live_snapshot.clone(), live_session.clone());
    }

    let mut session = live_session.clone();
    session.kills = 6;
    session.deaths = 1;

    let snapshot = match scenario {
        PreviewScenario::Live => unreachable!(),
        PreviewScenario::Hangar => GameSnapshot {
            phase: GamePhase::Hangar,
            captured_at: live_snapshot.captured_at,
            ..GameSnapshot::default()
        },
        PreviewScenario::Air => GameSnapshot {
            phase: GamePhase::Battle,
            battle_id: Some(BattleId::new()),
            vehicle: Some(Vehicle {
                technical_name: "j_7d".to_owned(),
                display_name: "J-7D".to_owned(),
                kind: VehicleKind::Aircraft,
            }),
            telemetry: Telemetry {
                speed_ias_kph: Some(1_080.0),
                speed_tas_kph: Some(1_145.0),
                altitude_agl_m: Some(75.0),
                altitude_msl_m: Some(1_420.0),
                ..Telemetry::default()
            },
            map: Some("Sinai".to_owned()),
            mode: Some("Air Realistic".to_owned()),
            captured_at: live_snapshot.captured_at,
        },
        PreviewScenario::Ground => GameSnapshot {
            phase: GamePhase::Battle,
            battle_id: Some(BattleId::new()),
            vehicle: Some(Vehicle {
                technical_name: "cn_al_khalid_i".to_owned(),
                display_name: "Al-Khalid-I".to_owned(),
                kind: VehicleKind::Ground,
            }),
            telemetry: Telemetry {
                speed_ground_kph: Some(48.0),
                crew_current: Some(3),
                crew_total: Some(3),
                ..Telemetry::default()
            },
            map: Some("Sinai".to_owned()),
            mode: Some("Ground Realistic".to_owned()),
            captured_at: live_snapshot.captured_at,
        },
    };

    (snapshot, session)
}
