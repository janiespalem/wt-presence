use wt_presence::{
    domain::{GamePhase, VehicleKind},
    state::{GameStateMachine, Observation},
};

fn observation(
    reachable: bool,
    map_valid: bool,
    vehicle_name: Option<&str>,
    vehicle_kind: VehicleKind,
    map_generation: Option<u64>,
) -> Observation {
    Observation {
        reachable,
        map_valid,
        vehicle_name: vehicle_name.map(str::to_owned),
        vehicle_kind,
        map_generation,
    }
}

#[test]
fn classifies_game_phases_from_local_telemetry() {
    let mut machine = GameStateMachine::default();

    let offline = machine.observe(observation(
        false,
        false,
        None,
        VehicleKind::Unknown,
        None,
    ));
    let hangar = machine.observe(observation(
        true,
        false,
        None,
        VehicleKind::Unknown,
        None,
    ));
    let loading = machine.observe(observation(
        true,
        true,
        None,
        VehicleKind::Unknown,
        Some(5),
    ));
    let battle = machine.observe(observation(
        true,
        true,
        Some("J-7D"),
        VehicleKind::Aircraft,
        Some(5),
    ));

    assert_eq!(offline.current.phase, GamePhase::Offline);
    assert_eq!(hangar.current.phase, GamePhase::Hangar);
    assert_eq!(loading.current.phase, GamePhase::Loading);
    assert_eq!(battle.current.phase, GamePhase::Battle);
    assert!(battle.current.battle_id.is_some());
}

#[test]
fn keeps_battle_identity_until_map_generation_changes() {
    let mut machine = GameStateMachine::default();

    let first = machine.observe(observation(
        true,
        true,
        Some("J-7D"),
        VehicleKind::Aircraft,
        Some(7),
    ));
    let same = machine.observe(observation(
        true,
        true,
        Some("J-7D"),
        VehicleKind::Aircraft,
        Some(7),
    ));
    let next = machine.observe(observation(
        true,
        true,
        Some("Al-Khalid-I"),
        VehicleKind::Ground,
        Some(8),
    ));

    assert_eq!(first.current.battle_id, same.current.battle_id);
    assert_ne!(same.current.battle_id, next.current.battle_id);
    assert!(next.battle_started);
}

#[test]
fn treats_dummy_vehicle_as_loading() {
    let mut machine = GameStateMachine::default();
    let transition = machine.observe(observation(
        true,
        true,
        Some("dummy_plane"),
        VehicleKind::Aircraft,
        Some(12),
    ));

    assert_eq!(transition.current.phase, GamePhase::Loading);
    assert!(transition.current.battle_id.is_none());
}

