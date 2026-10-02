use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tempfile::tempdir;
use tower::ServiceExt;
use wt_presence::{
    api::{ApiState, RuntimeStatus, router},
    config::{AppSettings, SettingsStore},
    domain::GameSnapshot,
    presence::PresencePreset,
    preview::{PreviewRequest, PreviewScenario},
    session::SessionEngine,
    storage::SessionRepository,
};

fn test_state(directory: &tempfile::TempDir) -> ApiState {
    let settings = AppSettings::default();
    ApiState::new(
        settings,
        SettingsStore::new(directory.path().join("settings.json")),
        SessionRepository::open_in_memory().unwrap(),
        "secret-token",
        "http://127.0.0.1:32147",
    )
}

#[tokio::test]
async fn rejects_requests_without_the_process_token() {
    let directory = tempdir().unwrap();
    let response = router(test_state(&directory))
        .oneshot(
            Request::builder()
                .uri("/api/v1/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_a_foreign_browser_origin_even_with_a_valid_token() {
    let directory = tempdir().unwrap();
    let response = router(test_state(&directory))
        .oneshot(
            Request::builder()
                .uri("/api/v1/status")
                .header("x-wt-presence-token", "secret-token")
                .header("origin", "https://evil.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn returns_structured_runtime_status_to_the_local_dashboard() {
    let directory = tempdir().unwrap();
    let state = test_state(&directory);
    state
        .set_status(RuntimeStatus {
            telemetry_connected: true,
            discord_connected: false,
            last_error: Some("Discord is not running".to_owned()),
            ..RuntimeStatus::default()
        })
        .await;

    let response = router(state)
        .oneshot(
            Request::builder()
                .uri("/api/v1/status")
                .header("x-wt-presence-token", "secret-token")
                .header("origin", "http://127.0.0.1:32147")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["telemetry_connected"], true);
    assert_eq!(json["discord_connected"], false);
    assert_eq!(json["last_error"], "Discord is not running");
}

#[tokio::test]
async fn previews_a_preset_against_the_current_snapshot() {
    let directory = tempdir().unwrap();
    let state = test_state(&directory);
    let snapshot = GameSnapshot::default();
    let session = SessionEngine::new(snapshot.captured_at).summary();
    state.set_game_state(snapshot, session).await;
    let request = PreviewRequest {
        preset: PresencePreset::minimal(),
        scenario: PreviewScenario::Air,
    };

    let response = router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/preview")
                .header("content-type", "application/json")
                .header("x-wt-presence-token", "secret-token")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["details"], "J-7D");
    assert_eq!(json["state"], "In battle");
}

#[tokio::test]
async fn invalid_settings_are_rejected_without_replacing_the_active_config() {
    let directory = tempdir().unwrap();
    let state = test_state(&directory);
    let settings = AppSettings {
        telemetry_url: "https://example.com".to_owned(),
        ..AppSettings::default()
    };

    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/settings")
                .header("content-type", "application/json")
                .header("x-wt-presence-token", "secret-token")
                .body(Body::from(serde_json::to_vec(&settings).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(state.settings().await, AppSettings::default());
}
