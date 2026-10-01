use crate::domain::{BattleId, GamePhase, GameSnapshot, Vehicle, VehicleKind};

#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub reachable: bool,
    pub map_valid: bool,
    pub vehicle_name: Option<String>,
    pub vehicle_kind: VehicleKind,
    pub map_generation: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StateTransition {
    pub previous: GamePhase,
    pub current: GameSnapshot,
    pub battle_started: bool,
    pub battle_ended: bool,
}

#[derive(Debug, Default)]
pub struct GameStateMachine {
    current: GameSnapshot,
    map_generation: Option<u64>,
}

impl GameStateMachine {
    pub fn observe(&mut self, observation: Observation) -> StateTransition {
        let previous = self.current.phase;
        let phase = classify(&observation);
        let generation_changed = observation.map_generation.is_some()
            && observation.map_generation != self.map_generation;

        let battle_id = if phase == GamePhase::Battle {
            if generation_changed || self.current.battle_id.is_none() {
                Some(BattleId::new())
            } else {
                self.current.battle_id
            }
        } else {
            None
        };

        if observation.map_generation.is_some() {
            self.map_generation = observation.map_generation;
        }

        let vehicle = observation.vehicle_name.as_ref().and_then(|name| {
            (phase == GamePhase::Battle).then(|| Vehicle {
                technical_name: name.clone(),
                display_name: humanize_vehicle_name(name),
                kind: observation.vehicle_kind,
            })
        });

        self.current = GameSnapshot {
            phase,
            battle_id,
            vehicle,
            ..GameSnapshot::default()
        };

        StateTransition {
            previous,
            current: self.current.clone(),
            battle_started: phase == GamePhase::Battle
                && (previous != GamePhase::Battle || generation_changed),
            battle_ended: previous == GamePhase::Battle && phase != GamePhase::Battle,
        }
    }
}

fn classify(observation: &Observation) -> GamePhase {
    if !observation.reachable {
        GamePhase::Offline
    } else if !observation.map_valid {
        GamePhase::Hangar
    } else if observation
        .vehicle_name
        .as_deref()
        .is_none_or(|name| name.starts_with("dummy_"))
    {
        GamePhase::Loading
    } else {
        GamePhase::Battle
    }
}

fn humanize_vehicle_name(value: &str) -> String {
    let value = value.rsplit('/').next().unwrap_or(value);
    value.replace('_', " ")
}

