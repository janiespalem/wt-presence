use std::{
    collections::HashSet,
    fs::{self, File},
    io::{BufReader, Write},
    net::IpAddr,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use thiserror::Error;
use url::Url;

use crate::presence::PresencePreset;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct AppSettings {
    pub schema_version: u32,
    pub telemetry_url: String,
    pub dashboard_port: u16,
    pub active_preset_id: String,
    pub presets: Vec<PresencePreset>,
    pub open_dashboard_on_start: bool,
    pub start_minimized: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: 2,
            telemetry_url: "http://127.0.0.1:8111".to_owned(),
            dashboard_port: 32147,
            active_preset_id: "minimal".to_owned(),
            presets: vec![PresencePreset::minimal()],
            open_dashboard_on_start: true,
            start_minimized: false,
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), SettingsError> {
        if self.schema_version != 2 {
            return Err(SettingsError::Invalid(format!(
                "unsupported settings schema version {}",
                self.schema_version
            )));
        }

        let telemetry_url = Url::parse(&self.telemetry_url)?;
        if telemetry_url.scheme() != "http" || !is_loopback_url(&telemetry_url) {
            return Err(SettingsError::Invalid(
                "telemetry URL must use HTTP on a loopback host".to_owned(),
            ));
        }

        if self.dashboard_port == 0 {
            return Err(SettingsError::Invalid(
                "dashboard port must be greater than zero".to_owned(),
            ));
        }

        let mut preset_ids = HashSet::new();
        for preset in &self.presets {
            if preset.id.trim().is_empty() {
                return Err(SettingsError::Invalid(
                    "presence preset IDs cannot be empty".to_owned(),
                ));
            }
            if !preset_ids.insert(preset.id.as_str()) {
                return Err(SettingsError::Invalid(format!(
                    "duplicate presence preset ID `{}`",
                    preset.id
                )));
            }
        }
        if !preset_ids.contains(self.active_preset_id.as_str()) {
            return Err(SettingsError::Invalid(format!(
                "active presence preset `{}` does not exist",
                self.active_preset_id
            )));
        }

        Ok(())
    }

    pub fn active_preset(&self) -> &PresencePreset {
        self.presets
            .iter()
            .find(|preset| preset.id == self.active_preset_id)
            .expect("validated settings always contain the active preset")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedSettings {
    pub settings: AppSettings,
    pub recovered_from: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn load_or_create(&self) -> Result<LoadedSettings, SettingsError> {
        if !self.path.exists() {
            let settings = AppSettings::default();
            self.save(&settings)?;
            return Ok(LoadedSettings {
                settings,
                recovered_from: None,
            });
        }

        let file = File::open(&self.path)?;
        let document = serde_json::from_reader::<_, serde_json::Value>(BufReader::new(file));
        let parsed_settings = match document {
            Ok(document) => {
                if let Some(version) = document
                    .get("schema_version")
                    .and_then(serde_json::Value::as_u64)
                    && !matches!(version, 1 | 2)
                {
                    return Err(SettingsError::Invalid(format!(
                        "unsupported settings schema version {version}"
                    )));
                }
                serde_json::from_value::<AppSettings>(document)
            }
            Err(error) => Err(error),
        };
        match parsed_settings {
            Ok(mut settings) => {
                let migrated = settings.schema_version == 1;
                if migrated {
                    settings.schema_version = 2;
                }
                settings.validate()?;
                if migrated {
                    self.save(&settings)?;
                }
                Ok(LoadedSettings {
                    settings,
                    recovered_from: None,
                })
            }
            Err(_) => {
                let backup = self.next_backup_path();
                fs::rename(&self.path, &backup)?;
                let settings = AppSettings::default();
                self.save(&settings)?;
                Ok(LoadedSettings {
                    settings,
                    recovered_from: Some(backup),
                })
            }
        }
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), SettingsError> {
        settings.validate()?;
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;

        let mut temporary = NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(&mut temporary, settings)?;
        temporary.write_all(b"\n")?;
        temporary.flush()?;
        temporary.as_file().sync_all()?;
        temporary.persist(&self.path).map_err(|error| error.error)?;

        Ok(())
    }

    fn next_backup_path(&self) -> PathBuf {
        let mut index = 0;
        loop {
            let suffix = if index == 0 {
                "corrupt".to_owned()
            } else {
                format!("corrupt-{index}")
            };
            let backup = self.path.with_extension(format!("{suffix}.json"));
            if !backup.exists() {
                return backup;
            }
            index += 1;
        }
    }
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("settings I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("settings JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid telemetry URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("invalid settings: {0}")]
    Invalid(String),
}

fn is_loopback_url(url: &Url) -> bool {
    match url.host_str() {
        Some("localhost") => true,
        Some(host) => host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback()),
        None => false,
    }
}
