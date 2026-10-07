use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;

use crate::{
    config::{AppSettings, SettingsError, SettingsStore},
    diagnostics::DiagnosticReport,
    domain::GameSnapshot,
    presence::{DiscordActivity, PresenceError},
    preview::{PreviewRequest, render_preview},
    session::{SessionEngine, SessionSummary},
    storage::{SessionRepository, StorageError, StoredSession},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RuntimeStatus {
    pub version: String,
    pub telemetry_connected: bool,
    pub discord_connected: bool,
    pub started_at: DateTime<Utc>,
    pub last_telemetry_at: Option<DateTime<Utc>>,
    pub last_discord_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl Default for RuntimeStatus {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            telemetry_connected: false,
            discord_connected: false,
            started_at: now,
            last_telemetry_at: None,
            last_discord_at: None,
            last_error: None,
            updated_at: now,
        }
    }
}

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

struct ApiStateInner {
    status: RwLock<RuntimeStatus>,
    snapshot: RwLock<GameSnapshot>,
    session: RwLock<SessionSummary>,
    presence: RwLock<Option<DiscordActivity>>,
    settings: RwLock<AppSettings>,
    settings_store: SettingsStore,
    sessions: SessionRepository,
    token: String,
    allowed_origin: String,
}

impl ApiState {
    pub fn new(
        settings: AppSettings,
        settings_store: SettingsStore,
        sessions: SessionRepository,
        token: impl Into<String>,
        allowed_origin: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            inner: Arc::new(ApiStateInner {
                status: RwLock::new(RuntimeStatus::default()),
                snapshot: RwLock::new(GameSnapshot::default()),
                session: RwLock::new(SessionEngine::new(now).summary()),
                presence: RwLock::new(None),
                settings: RwLock::new(settings),
                settings_store,
                sessions,
                token: token.into(),
                allowed_origin: allowed_origin.into(),
            }),
        }
    }

    pub async fn set_status(&self, mut status: RuntimeStatus) {
        status.updated_at = Utc::now();
        let mut current = self.inner.status.write().await;
        if current.telemetry_connected != status.telemetry_connected
            || current.discord_connected != status.discord_connected
            || current.last_error.is_some() != status.last_error.is_some()
        {
            tracing::info!(
                telemetry_connected = status.telemetry_connected,
                discord_connected = status.discord_connected,
                has_error = status.last_error.is_some(),
                "connection state changed"
            );
        }
        *current = status;
    }

    pub async fn set_game_state(&self, snapshot: GameSnapshot, session: SessionSummary) {
        *self.inner.snapshot.write().await = snapshot;
        *self.inner.session.write().await = session;
    }

    pub async fn set_presence(&self, presence: Option<DiscordActivity>) {
        *self.inner.presence.write().await = presence;
    }

    pub async fn settings(&self) -> AppSettings {
        self.inner.settings.read().await.clone()
    }

    pub async fn status(&self) -> RuntimeStatus {
        self.inner.status.read().await.clone()
    }

    pub async fn snapshot(&self) -> GameSnapshot {
        self.inner.snapshot.read().await.clone()
    }

    pub fn persist_session(&self, id: &str, session: &SessionSummary) -> Result<(), StorageError> {
        self.inner.sessions.save(id, session)
    }

    pub fn recent_sessions(&self, limit: usize) -> Result<Vec<StoredSession>, StorageError> {
        self.inner.sessions.recent(limit)
    }
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/api/v1/status", get(get_status))
        .route("/api/v1/diagnostics", get(get_diagnostics))
        .route("/api/v1/snapshot", get(get_snapshot))
        .route("/api/v1/settings", get(get_settings).put(put_settings))
        .route("/api/v1/preview", post(post_preview))
        .route("/api/v1/presence/state", get(get_presence))
        .route("/api/v1/sessions", get(get_sessions))
        .route_layer(middleware::from_fn_with_state(state.clone(), authorize))
        .with_state(state)
}

async fn authorize(
    State(state): State<ApiState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let token_matches = headers
        .get("x-wt-presence-token")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == state.inner.token);
    if !token_matches {
        return api_error(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "missing or invalid process token",
        );
    }

    let origin_allowed = match headers.get("origin") {
        None => true,
        Some(value) => value
            .to_str()
            .is_ok_and(|value| value == state.inner.allowed_origin),
    };
    if !origin_allowed {
        return api_error(
            StatusCode::FORBIDDEN,
            "forbidden_origin",
            "browser origin is not allowed",
        );
    }

    next.run(request).await
}

async fn get_status(State(state): State<ApiState>) -> Json<RuntimeStatus> {
    Json(state.inner.status.read().await.clone())
}

async fn get_diagnostics(State(state): State<ApiState>) -> Json<DiagnosticReport> {
    let status = state.inner.status.read().await;
    let snapshot = state.inner.snapshot.read().await;
    Json(DiagnosticReport::capture(&status, &snapshot))
}

async fn get_snapshot(State(state): State<ApiState>) -> Json<GameSnapshot> {
    Json(state.inner.snapshot.read().await.clone())
}

async fn get_settings(State(state): State<ApiState>) -> Json<AppSettings> {
    Json(state.inner.settings.read().await.clone())
}

async fn put_settings(
    State(state): State<ApiState>,
    Json(settings): Json<AppSettings>,
) -> Result<Json<AppSettings>, ApiError> {
    settings.validate().map_err(ApiError::settings)?;
    let mut current = state.inner.settings.write().await;
    state
        .inner
        .settings_store
        .save(&settings)
        .map_err(ApiError::settings)?;
    *current = settings.clone();
    Ok(Json(settings))
}

async fn post_preview(
    State(state): State<ApiState>,
    Json(request): Json<PreviewRequest>,
) -> Result<Json<DiscordActivity>, ApiError> {
    let snapshot = state.inner.snapshot.read().await.clone();
    let session = state.inner.session.read().await.clone();
    let activity = render_preview(&snapshot, &session, &request.preset, request.scenario)
        .map_err(ApiError::presence)?;
    Ok(Json(activity))
}

async fn get_presence(State(state): State<ApiState>) -> Json<Option<DiscordActivity>> {
    Json(state.inner.presence.read().await.clone())
}

#[derive(Debug, Deserialize)]
struct SessionsQuery {
    limit: Option<usize>,
}

async fn get_sessions(
    State(state): State<ApiState>,
    Query(query): Query<SessionsQuery>,
) -> Result<Json<Vec<StoredSession>>, ApiError> {
    let sessions = state
        .inner
        .sessions
        .recent(query.limit.unwrap_or(20).min(100))
        .map_err(ApiError::storage)?;
    Ok(Json(sessions))
}

struct ApiError {
    status: StatusCode,
    kind: &'static str,
    message: String,
}

impl ApiError {
    fn settings(error: SettingsError) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            kind: "invalid_settings",
            message: error.to_string(),
        }
    }

    fn presence(error: PresenceError) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            kind: error.kind(),
            message: error.to_string(),
        }
    }

    fn storage(error: StorageError) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            kind: "storage",
            message: error.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        api_error(self.status, self.kind, &self.message)
    }
}

fn api_error(status: StatusCode, kind: &'static str, message: &str) -> Response {
    (
        status,
        Json(json!({ "error": { "kind": kind, "message": message } })),
    )
        .into_response()
}
