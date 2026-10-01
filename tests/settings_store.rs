use std::fs;

use tempfile::tempdir;
use wt_presence::{
    config::{AppSettings, SettingsError, SettingsStore},
    presence::PresencePreset,
};

#[test]
fn creates_a_useful_default_config_on_first_run() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let store = SettingsStore::new(&path);

    let loaded = store.load_or_create().unwrap();

    assert!(path.exists());
    assert_eq!(loaded.settings.schema_version, 1);
    assert_eq!(loaded.settings.telemetry_url, "http://127.0.0.1:8111");
    assert_eq!(loaded.settings.active_preset_id, "minimal");
    assert!(loaded.recovered_from.is_none());
}

#[test]
fn saves_settings_atomically_and_round_trips_presets() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    let store = SettingsStore::new(&path);
    let mut settings = AppSettings {
        active_preset_id: "custom".to_owned(),
        ..AppSettings::default()
    };
    settings.presets.push(PresencePreset {
        id: "custom".to_owned(),
        name: "Custom".to_owned(),
        details_template: "{{ vehicle.name }}".to_owned(),
        state_template: "{{ game.mode }}".to_owned(),
        large_image: None,
        small_image: None,
        show_elapsed: true,
    });

    store.save(&settings).unwrap();
    let loaded = store.load_or_create().unwrap();

    assert_eq!(loaded.settings, settings);
    assert!(!directory.path().join("settings.json.tmp").exists());
}

#[test]
fn backs_up_corrupt_json_and_recovers_defaults() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    fs::write(&path, "{ definitely not json").unwrap();
    let store = SettingsStore::new(&path);

    let loaded = store.load_or_create().unwrap();

    let backup = loaded.recovered_from.expect("corrupt file backup");
    assert!(backup.exists());
    assert_eq!(fs::read_to_string(backup).unwrap(), "{ definitely not json");
    assert_eq!(loaded.settings, AppSettings::default());
    assert!(serde_json::from_str::<AppSettings>(&fs::read_to_string(path).unwrap()).is_ok());
}

#[test]
fn rejects_remote_telemetry_sources() {
    let directory = tempdir().unwrap();
    let store = SettingsStore::new(directory.path().join("settings.json"));
    let settings = AppSettings {
        telemetry_url: "https://example.com:8111".to_owned(),
        ..AppSettings::default()
    };

    let error = store.save(&settings).unwrap_err();

    assert!(matches!(error, SettingsError::Invalid(_)));
    assert!(error.to_string().contains("loopback"));
}
