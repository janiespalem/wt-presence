use serde_json::json;
use wt_presence::{
    domain::VehicleKind,
    telemetry::{PayloadSet, normalize_payloads},
};

#[test]
fn normalizes_aircraft_payloads() {
    let normalized = normalize_payloads(PayloadSet {
        indicators: json!({
            "valid": true,
            "army": "air",
            "type": "j_7d",
            "radio_altitude": 31.4
        }),
        state: json!({
            "valid": true,
            "IAS, km/h": 912.5,
            "TAS, km/h": 978.2,
            "H, m": 465.0
        }),
        map_info: json!({
            "valid": true,
            "map_generation": 42,
            "map_name": "Sinai"
        }),
    });

    assert!(normalized.observation.reachable);
    assert!(normalized.observation.map_valid);
    assert_eq!(normalized.observation.vehicle_kind, VehicleKind::Aircraft);
    assert_eq!(normalized.observation.vehicle_name.as_deref(), Some("j_7d"));
    assert_eq!(normalized.observation.map_generation, Some(42));
    assert_eq!(normalized.telemetry.speed_ias_kph, Some(912.5));
    assert_eq!(normalized.telemetry.speed_tas_kph, Some(978.2));
    assert_eq!(normalized.telemetry.altitude_agl_m, Some(31.4));
    assert_eq!(normalized.telemetry.altitude_msl_m, Some(465.0));
    assert_eq!(normalized.map.as_deref(), Some("Sinai"));
}

#[test]
fn normalizes_ground_payloads_with_string_numbers() {
    let normalized = normalize_payloads(PayloadSet {
        indicators: json!({
            "valid": true,
            "army": "tank",
            "type": "tankModels/cn_al_khalid_i",
            "speed": "54.8",
            "crew_total": 3,
            "crew_current": 2
        }),
        state: json!({"valid": true}),
        map_info: json!({"valid": true, "map_generation": 9}),
    });

    assert_eq!(normalized.observation.vehicle_kind, VehicleKind::Ground);
    assert_eq!(
        normalized.observation.vehicle_name.as_deref(),
        Some("tankModels/cn_al_khalid_i")
    );
    assert_eq!(normalized.telemetry.speed_ground_kph, Some(54.8));
    assert_eq!(normalized.telemetry.crew_current, Some(2));
    assert_eq!(normalized.telemetry.crew_total, Some(3));
}

#[test]
fn invalid_reachable_payload_is_hangar_observation() {
    let normalized = normalize_payloads(PayloadSet {
        indicators: json!({"valid": false}),
        state: json!({"valid": false}),
        map_info: json!({"valid": false}),
    });

    assert!(normalized.observation.reachable);
    assert!(!normalized.observation.map_valid);
    assert!(normalized.observation.vehicle_name.is_none());
}

#[test]
fn malformed_optional_values_are_ignored() {
    let normalized = normalize_payloads(PayloadSet {
        indicators: json!({
            "valid": true,
            "army": "air",
            "type": 42,
            "radio_altitude": "not-a-number"
        }),
        state: json!({"IAS, km/h": []}),
        map_info: json!({"valid": true, "map_generation": "bad"}),
    });

    assert!(normalized.observation.vehicle_name.is_none());
    assert!(normalized.observation.map_generation.is_none());
    assert!(normalized.telemetry.speed_ias_kph.is_none());
    assert!(normalized.telemetry.altitude_agl_m.is_none());
}

