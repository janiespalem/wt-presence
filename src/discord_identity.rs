use crate::domain::{GamePhase, GameSnapshot, VehicleKind};

pub struct DiscordIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresenceArtwork {
    Default,
    Air,
    Ground,
    Naval,
    Hangar,
}

impl DiscordIdentity {
    pub fn application_id() -> &'static str {
        "1555607328965926974"
    }

    pub fn asset_key(artwork: PresenceArtwork) -> &'static str {
        match artwork {
            PresenceArtwork::Default => "presence-default",
            PresenceArtwork::Air => "presence-air",
            PresenceArtwork::Ground => "presence-ground",
            PresenceArtwork::Naval => "presence-naval",
            PresenceArtwork::Hangar => "presence-hangar",
        }
    }

    pub fn artwork_for(snapshot: &GameSnapshot) -> PresenceArtwork {
        match snapshot.phase {
            GamePhase::Offline | GamePhase::Loading => PresenceArtwork::Default,
            GamePhase::Hangar => PresenceArtwork::Hangar,
            GamePhase::Battle => match snapshot.vehicle.as_ref().map(|vehicle| vehicle.kind) {
                Some(VehicleKind::Aircraft) => PresenceArtwork::Air,
                Some(VehicleKind::Ground) => PresenceArtwork::Ground,
                Some(VehicleKind::Naval) => PresenceArtwork::Naval,
                Some(VehicleKind::Unknown) | None => PresenceArtwork::Default,
            },
        }
    }
}
