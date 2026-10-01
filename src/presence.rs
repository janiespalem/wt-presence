use minijinja::{Environment, UndefinedBehavior};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{
    domain::{GamePhase, GameSnapshot},
    session::SessionSummary,
};

const DISCORD_TEXT_LIMIT: usize = 128;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PresencePreset {
    pub id: String,
    pub name: String,
    pub details_template: String,
    pub state_template: String,
    pub large_image: Option<String>,
    pub small_image: Option<String>,
    pub show_elapsed: bool,
}

impl PresencePreset {
    pub fn minimal() -> Self {
        Self {
            id: "minimal".to_owned(),
            name: "Minimal".to_owned(),
            details_template:
                "{% if vehicle %}{{ vehicle.name }}{% else %}War Thunder{% endif %}"
                    .to_owned(),
            state_template: "{{ game.phase_label }}".to_owned(),
            large_image: Some("war_thunder".to_owned()),
            small_image: None,
            show_elapsed: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DiscordActivity {
    pub details: Option<String>,
    pub state: Option<String>,
    pub large_image: Option<String>,
    pub small_image: Option<String>,
    pub started_at: Option<i64>,
}

#[derive(Debug, Error)]
pub enum PresenceError {
    #[error("presence {field} template error in `{template}`: {source}")]
    Template {
        field: &'static str,
        template: String,
        #[source]
        source: minijinja::Error,
    },
    #[error("Discord {field} is {actual} characters; maximum is {max}")]
    FieldTooLong {
        field: &'static str,
        max: usize,
        actual: usize,
    },
}

impl PresenceError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Template { .. } => "template",
            Self::FieldTooLong { .. } => "field_too_long",
        }
    }
}

#[derive(Debug, Default)]
pub struct PresenceRenderer;

impl PresenceRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        snapshot: &GameSnapshot,
        session: &SessionSummary,
        preset: &PresencePreset,
    ) -> Result<DiscordActivity, PresenceError> {
        let mut environment = Environment::new();
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        environment.add_filter("round", |value: f64| value.round() as i64);
        let context = template_context(snapshot, session);

        let details = render_field(
            &environment,
            "details",
            &preset.details_template,
            &context,
        )?;
        let state = render_field(
            &environment,
            "state",
            &preset.state_template,
            &context,
        )?;

        Ok(DiscordActivity {
            details,
            state,
            large_image: preset.large_image.clone(),
            small_image: preset.small_image.clone(),
            started_at: preset.show_elapsed.then(|| snapshot.captured_at.timestamp()),
        })
    }
}

fn render_field(
    environment: &Environment<'_>,
    field: &'static str,
    template: &str,
    context: &serde_json::Value,
) -> Result<Option<String>, PresenceError> {
    let rendered = environment
        .render_str(template, context)
        .map_err(|source| PresenceError::Template {
            field,
            template: template.to_owned(),
            source,
        })?
        .trim()
        .to_owned();
    let actual = rendered.chars().count();
    if actual > DISCORD_TEXT_LIMIT {
        return Err(PresenceError::FieldTooLong {
            field,
            max: DISCORD_TEXT_LIMIT,
            actual,
        });
    }

    Ok((!rendered.is_empty()).then_some(rendered))
}

fn template_context(snapshot: &GameSnapshot, session: &SessionSummary) -> serde_json::Value {
    let vehicle = snapshot.vehicle.as_ref().map(|vehicle| {
        json!({
            "name": vehicle.display_name,
            "technical_name": vehicle.technical_name,
            "kind": vehicle.kind,
        })
    });

    json!({
        "game": {
            "phase": snapshot.phase,
            "phase_label": phase_label(snapshot.phase),
            "mode": snapshot.mode,
            "map": snapshot.map,
        },
        "vehicle": vehicle,
        "telemetry": {
            "ias": snapshot.telemetry.speed_ias_kph,
            "tas": snapshot.telemetry.speed_tas_kph,
            "agl": snapshot.telemetry.altitude_agl_m,
            "msl": snapshot.telemetry.altitude_msl_m,
            "ground_speed": snapshot.telemetry.speed_ground_kph,
            "crew_current": snapshot.telemetry.crew_current,
            "crew_total": snapshot.telemetry.crew_total,
        },
        "session": {
            "started_at": session.started_at.timestamp(),
            "kills": session.kills,
            "deaths": session.deaths,
            "battles": session.completed_battles.len(),
        },
    })
}

fn phase_label(phase: GamePhase) -> &'static str {
    match phase {
        GamePhase::Offline => "Offline",
        GamePhase::Hangar => "In hangar",
        GamePhase::Loading => "Loading battle",
        GamePhase::Battle => "In battle",
    }
}
