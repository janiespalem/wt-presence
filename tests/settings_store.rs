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
    assert_eq!(loaded.settings.schema_version, 2);
    assert_eq!(loaded.settings.telemetry_url, "http://127.0.0.1:8111");
    assert_eq!(loaded.settings.active_preset_id, "minimal");
    assert!(loaded.recovered_from.is_none());
}

#[test]
fn migrates_schema_one_without_losing_user_configuration() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("settings.json");
    fs::write(
        &path,
        r#"{
            "schema_version": 1,
            "telemetry_url": "http://localhost:8222",
            "dashboard_port": 32222,
            "discord_application_id": "999",
            "active_preset_id": "custom",
            "presets": [
                {
                    "id": "spare",
                    "name": "Spare preset",
                    "details_template": "War Thunder",
                    "state_template": "{{ game.phase_label }}",
                    "large_image": null,
                    "small_image": null,
                    "show_elapsed": false
                },
                {
                    "id": "custom",
                    "name": "My custom preset",
                    "details_template": "Flying {{ vehicle.name }}",
                    "state_template": "{{ game.mode }} | {{ game.phase_label }}",
                    "large_image": "my_large_asset",
                    "small_image": "my_small_asset",
                    "show_elapsed": true
                }
            ],
            "open_dashboard_on_start": false,
            "start_minimized": true
        }"#,
    )
    .unwrap();
    let store = SettingsStore::new(&path);

    let loaded = store.load_or_create().unwrap();

    assert_eq!(loaded.settings.schema_version, 2);
    assert_eq!(loaded.settings.telemetry_url, "http://localhost:8222");
    assert_eq!(loaded.settings.dashboard_port, 32222);
    assert_eq!(loaded.settings.active_preset_id, "custom");
    assert_eq!(
        loaded.settings.presets,
        vec![
            PresencePreset {
                id: "spare".to_owned(),
                name: "Spare preset".to_owned(),
                details_template: "War Thunder".to_owned(),
                state_template: "{{ game.phase_label }}".to_owned(),
                large_image: None,
                small_image: None,
                show_elapsed: false,
            },
            PresencePreset {
                id: "custom".to_owned(),
                name: "My custom preset".to_owned(),
                details_template: "Flying {{ vehicle.name }}".to_owned(),
                state_template: "{{ game.mode }} | {{ game.phase_label }}".to_owned(),
                large_image: Some("my_large_asset".to_owned()),
                small_image: Some("my_small_asset".to_owned()),
                show_elapsed: true,
            },
        ]
    );
    assert!(!loaded.settings.open_dashboard_on_start);
    assert!(loaded.settings.start_minimized);
    assert!(loaded.recovered_from.is_none());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);

    let persisted = fs::read_to_string(&path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&persisted).unwrap();
    assert_eq!(json["schema_version"], 2);
    assert!(json.get("discord_application_id").is_none());

    let reloaded = store.load_or_create().unwrap();
    assert_eq!(reloaded, loaded);
    assert_eq!(fs::read_to_string(&path).unwrap(), persisted);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn rejects_future_schema_without_recovering_or_rewriting_settings() {
    for original in [
        r#"{"schema_version": 42, "future_preference": "keep me"}"#,
        r#"{"schema_version": 42, "telemetry_url": {"host": "localhost", "port": 8222}}"#,
    ] {
        let directory = tempdir().unwrap();
        let path = directory.path().join("settings.json");
        fs::write(&path, original).unwrap();
        let store = SettingsStore::new(&path);

        let error = store.load_or_create().unwrap_err();

        assert!(matches!(error, SettingsError::Invalid(_)));
        assert_eq!(fs::read_to_string(path).unwrap(), original);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
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

#[test]
fn minimized_start_never_opens_the_dashboard() {
    for open in [false, true] {
        for minimized in [false, true] {
            let settings = wt_presence::config::AppSettings {
                open_dashboard_on_start: open,
                start_minimized: minimized,
                ..Default::default()
            };
            assert_eq!(settings.should_open_dashboard(), open && !minimized);
        }
    }
}
