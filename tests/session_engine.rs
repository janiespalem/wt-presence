use chrono::{TimeZone, Utc};
use wt_presence::{
    domain::{BattleId, GamePhase, GameSnapshot, Vehicle, VehicleKind},
    session::{CombatEvent, CombatEventKind, SessionEngine},
};

fn at(second: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(second, 0).unwrap()
}

fn battle_snapshot(id: BattleId, second: i64, vehicle_name: &str) -> GameSnapshot {
    GameSnapshot {
        phase: GamePhase::Battle,
        battle_id: Some(id),
        vehicle: Some(Vehicle {
            technical_name: vehicle_name.to_owned(),
            display_name: vehicle_name.to_owned(),
            kind: VehicleKind::Aircraft,
        }),
        captured_at: at(second),
        ..GameSnapshot::default()
    }
}

#[test]
fn completes_a_battle_when_game_returns_to_hangar() {
    let id = BattleId::new();
    let mut engine = SessionEngine::new(at(100));

    engine.apply(&battle_snapshot(id, 110, "J-7D"), &[]);
    engine.apply(&battle_snapshot(id, 120, "J-7D"), &[]);
    let summary = engine.apply(
        &GameSnapshot {
            phase: GamePhase::Hangar,
            captured_at: at(140),
            ..GameSnapshot::default()
        },
        &[],
    );

    assert!(summary.current_battle.is_none());
    assert_eq!(summary.completed_battles.len(), 1);
    assert_eq!(summary.completed_battles[0].id, id);
    assert_eq!(summary.completed_battles[0].duration_seconds(), 30);
    assert_eq!(summary.completed_battles[0].vehicles.len(), 1);
}

#[test]
fn deduplicates_combat_events_by_id() {
    let id = BattleId::new();
    let mut engine = SessionEngine::new(at(100));
    let event = CombatEvent {
        id: "hud-44".to_owned(),
        kind: CombatEventKind::Kill,
        occurred_at: at(115),
    };

    engine.apply(
        &battle_snapshot(id, 110, "J-7D"),
        std::slice::from_ref(&event),
    );
    let summary = engine.apply(&battle_snapshot(id, 120, "J-7D"), &[event]);

    assert_eq!(summary.kills, 1);
    assert_eq!(summary.current_battle.unwrap().kills, Some(1));
}

#[test]
fn ignores_combat_events_outside_a_battle() {
    let mut engine = SessionEngine::new(at(100));
    let event = CombatEvent {
        id: "stale-event".to_owned(),
        kind: CombatEventKind::Death,
        occurred_at: at(105),
    };

    let summary = engine.apply(
        &GameSnapshot {
            phase: GamePhase::Hangar,
            captured_at: at(110),
            ..GameSnapshot::default()
        },
        &[event],
    );

    assert_eq!(summary.deaths, 0);
    assert!(summary.completed_battles.is_empty());
}

#[test]
fn records_each_vehicle_used_during_one_battle_once() {
    let id = BattleId::new();
    let mut engine = SessionEngine::new(at(100));

    engine.apply(&battle_snapshot(id, 110, "J-7D"), &[]);
    engine.apply(&battle_snapshot(id, 120, "J-7D"), &[]);
    let summary = engine.apply(&battle_snapshot(id, 130, "Al-Khalid-I"), &[]);

    let battle = summary.current_battle.unwrap();
    assert_eq!(battle.vehicles.len(), 2);
    assert_eq!(battle.vehicles[0].display_name, "J-7D");
    assert_eq!(battle.vehicles[1].display_name, "Al-Khalid-I");
}
