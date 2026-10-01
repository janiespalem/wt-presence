use axum::{Json, Router, routing::get};
use serde_json::json;
use tokio::net::TcpListener;
use wt_presence::telemetry::WtTelemetryClient;

async fn spawn_fixture_server() -> String {
    let app = Router::new()
        .route(
            "/indicators",
            get(|| async {
                Json(json!({
                    "valid": true,
                    "army": "air",
                    "type": "j_7d",
                    "radio_altitude": 31
                }))
            }),
        )
        .route(
            "/state",
            get(|| async { Json(json!({"valid": true, "IAS, km/h": 900})) }),
        )
        .route(
            "/map_info.json",
            get(|| async { Json(json!({"valid": true, "map_generation": 4})) }),
        );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{address}")
}

#[tokio::test]
async fn polls_all_required_war_thunder_endpoints() {
    let base_url = spawn_fixture_server().await;
    let client = WtTelemetryClient::new(base_url).unwrap();

    let payloads = client.poll_raw().await.unwrap();

    assert_eq!(payloads.indicators["type"], "j_7d");
    assert_eq!(payloads.state["IAS, km/h"], 900);
    assert_eq!(payloads.map_info["map_generation"], 4);
}

#[tokio::test]
async fn reports_unreachable_game_without_panicking() {
    let client = WtTelemetryClient::new("http://127.0.0.1:9".to_owned()).unwrap();

    let error = client.poll_raw().await.unwrap_err();

    assert_eq!(error.kind(), "unreachable");
}

