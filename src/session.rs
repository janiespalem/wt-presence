use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{BattleId, GamePhase, GameSnapshot, Vehicle};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CombatEventKind {
    Kill,
    Death,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CombatEvent {
    pub id: String,
    pub kind: CombatEventKind,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BattleSummary {
    pub id: BattleId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub vehicles: Vec<Vehicle>,
    pub kills: Option<u32>,
    pub deaths: Option<u32>,
}

impl BattleSummary {
    pub fn duration_seconds(&self) -> i64 {
        self.ended_at
            .unwrap_or_else(Utc::now)
            .signed_duration_since(self.started_at)
            .num_seconds()
            .max(0)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SessionSummary {
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub kills: u32,
    pub deaths: u32,
    pub current_battle: Option<BattleSummary>,
    pub completed_battles: Vec<BattleSummary>,
}

#[derive(Debug)]
pub struct SessionEngine {
    summary: SessionSummary,
    seen_events: HashSet<String>,
}

impl SessionEngine {
    pub fn new(started_at: DateTime<Utc>) -> Self {
        Self {
            summary: SessionSummary {
                started_at,
                updated_at: started_at,
                kills: 0,
                deaths: 0,
                current_battle: None,
                completed_battles: Vec::new(),
            },
            seen_events: HashSet::new(),
        }
    }

    pub fn apply(
        &mut self,
        snapshot: &GameSnapshot,
        events: &[CombatEvent],
    ) -> SessionSummary {
        self.summary.updated_at = snapshot.captured_at;

        if snapshot.phase == GamePhase::Battle {
            if let Some(id) = snapshot.battle_id {
                let changed = self
                    .summary
                    .current_battle
                    .as_ref()
                    .is_some_and(|battle| battle.id != id);
                if changed {
                    self.finish_current_battle(snapshot.captured_at);
                }
                if self.summary.current_battle.is_none() {
                    self.summary.current_battle = Some(BattleSummary {
                        id,
                        started_at: snapshot.captured_at,
                        ended_at: None,
                        vehicles: Vec::new(),
                        kills: Some(0),
                        deaths: Some(0),
                    });
                }
                self.record_vehicle(snapshot.vehicle.as_ref());
                self.record_events(events);
            }
        } else {
            self.finish_current_battle(snapshot.captured_at);
        }

        self.summary.clone()
    }

    pub fn summary(&self) -> SessionSummary {
        self.summary.clone()
    }

    fn record_vehicle(&mut self, vehicle: Option<&Vehicle>) {
        let Some(vehicle) = vehicle else {
            return;
        };
        let Some(battle) = self.summary.current_battle.as_mut() else {
            return;
        };
        if !battle
            .vehicles
            .iter()
            .any(|known| known.technical_name == vehicle.technical_name)
        {
            battle.vehicles.push(vehicle.clone());
        }
    }

    fn record_events(&mut self, events: &[CombatEvent]) {
        let Some(battle) = self.summary.current_battle.as_mut() else {
            return;
        };
        for event in events {
            if !self.seen_events.insert(event.id.clone()) {
                continue;
            }
            match event.kind {
                CombatEventKind::Kill => {
                    self.summary.kills += 1;
                    battle.kills = battle.kills.map(|kills| kills + 1);
                }
                CombatEventKind::Death => {
                    self.summary.deaths += 1;
                    battle.deaths = battle.deaths.map(|deaths| deaths + 1);
                }
            }
        }
    }

    fn finish_current_battle(&mut self, ended_at: DateTime<Utc>) {
        if let Some(mut battle) = self.summary.current_battle.take() {
            battle.ended_at = Some(ended_at);
            self.summary.completed_battles.push(battle);
        }
    }
}

