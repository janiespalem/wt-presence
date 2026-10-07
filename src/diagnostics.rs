use std::{fs, io, path::Path};

use chrono::{DateTime, Days, NaiveDate, Utc};
use serde::Serialize;

use crate::{
    api::RuntimeStatus,
    domain::{GamePhase, GameSnapshot},
};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiagnosticReport {
    pub version: String,
    pub platform: &'static str,
    pub architecture: &'static str,
    pub telemetry_connected: bool,
    pub discord_connected: bool,
    pub phase: GamePhase,
    pub started_at: DateTime<Utc>,
    pub last_telemetry_at: Option<DateTime<Utc>>,
    pub last_discord_at: Option<DateTime<Utc>>,
    pub has_error: bool,
    pub updated_at: DateTime<Utc>,
}

impl DiagnosticReport {
    pub fn capture(status: &RuntimeStatus, snapshot: &GameSnapshot) -> Self {
        Self {
            version: status.version.clone(),
            platform: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
            telemetry_connected: status.telemetry_connected,
            discord_connected: status.discord_connected,
            phase: snapshot.phase,
            started_at: status.started_at,
            last_telemetry_at: status.last_telemetry_at,
            last_discord_at: status.last_discord_at,
            has_error: status.last_error.is_some(),
            updated_at: status.updated_at,
        }
    }
}

pub fn prune_old_logs(
    directory: &Path,
    today: NaiveDate,
    retention_days: u64,
) -> io::Result<usize> {
    let cutoff = today
        .checked_sub_days(Days::new(retention_days.saturating_sub(1)))
        .unwrap_or(NaiveDate::MIN);
    let mut removed = 0;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(date) = name.strip_prefix("wt-presence.log.") else {
            continue;
        };
        let Ok(date) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
            continue;
        };
        if date < cutoff && entry.file_type()?.is_file() {
            fs::remove_file(entry.path())?;
            removed += 1;
        }
    }
    Ok(removed)
}
