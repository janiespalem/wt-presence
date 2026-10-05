use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, Assets, Timestamps},
};
use uuid::Uuid;

use crate::{
    api::{ApiState, RuntimeStatus},
    discord_identity::DiscordIdentity,
    domain::{GamePhase, Telemetry},
    presence::{DiscordActivity, PresenceRenderer},
    session::SessionEngine,
    state::{GameStateMachine, Observation},
    telemetry::{NormalizedTelemetry, WtTelemetryClient, normalize_payloads},
};

#[async_trait]
pub trait TelemetrySource: Send + Sync {
    async fn poll(&self) -> Result<NormalizedTelemetry, String>;
}

#[async_trait]
impl TelemetrySource for WtTelemetryClient {
    async fn poll(&self) -> Result<NormalizedTelemetry, String> {
        self.poll_raw()
            .await
            .map(normalize_payloads)
            .map_err(|error| error.to_string())
    }
}

pub trait PresenceSink: Send {
    fn publish(&mut self, activity: &DiscordActivity) -> Result<(), String>;
    fn clear(&mut self) -> Result<(), String>;
}

impl<T: PresenceSink + ?Sized> PresenceSink for Box<T> {
    fn publish(&mut self, activity: &DiscordActivity) -> Result<(), String> {
        (**self).publish(activity)
    }

    fn clear(&mut self) -> Result<(), String> {
        (**self).clear()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublishOutcome {
    Published,
    Skipped,
}

pub struct PresencePublisher<S> {
    inner: S,
    refresh_interval: Duration,
    last_published: Option<(DiscordActivity, DateTime<Utc>)>,
}

impl<S: PresenceSink> PresencePublisher<S> {
    pub fn new(inner: S, refresh_interval: Duration) -> Self {
        Self {
            inner,
            refresh_interval,
            last_published: None,
        }
    }

    pub fn publish(
        &mut self,
        activity: &DiscordActivity,
        now: DateTime<Utc>,
    ) -> Result<PublishOutcome, String> {
        if self.last_published.as_ref().is_some_and(|(previous, at)| {
            previous == activity
                && now
                    .signed_duration_since(*at)
                    .to_std()
                    .unwrap_or_default()
                    < self.refresh_interval
        }) {
            return Ok(PublishOutcome::Skipped);
        }
        self.inner.publish(activity)?;
        self.last_published = Some((activity.clone(), now));
        Ok(PublishOutcome::Published)
    }

    pub fn clear(&mut self) -> Result<(), String> {
        self.last_published = None;
        self.inner.clear()
    }
}

pub struct RuntimeEngine<T, S> {
    telemetry: T,
    presence: PresencePublisher<S>,
    api: ApiState,
    game: GameStateMachine,
    session: SessionEngine,
    session_id: String,
    renderer: PresenceRenderer,
}

impl<T, S> RuntimeEngine<T, S>
where
    T: TelemetrySource,
    S: PresenceSink,
{
    pub fn new(telemetry: T, presence: S, api: ApiState, started_at: DateTime<Utc>) -> Self {
        Self {
            telemetry,
            presence: PresencePublisher::new(presence, Duration::from_secs(15)),
            api,
            game: GameStateMachine::default(),
            session: SessionEngine::new(started_at),
            session_id: Uuid::new_v4().to_string(),
            renderer: PresenceRenderer::new(),
        }
    }

    pub async fn tick(&mut self, now: DateTime<Utc>) {
        match self.telemetry.poll().await {
            Ok(telemetry) => self.handle_telemetry(telemetry, now).await,
            Err(error) => self.handle_unreachable(error, now).await,
        }
    }

    async fn handle_telemetry(&mut self, telemetry: NormalizedTelemetry, now: DateTime<Utc>) {
        let mut snapshot = self.game.observe(telemetry.observation).current;
        snapshot.telemetry = telemetry.telemetry;
        snapshot.map = telemetry.map;
        snapshot.mode = telemetry.mode;
        snapshot.captured_at = now;

        let session = self.session.apply(&snapshot, &[]);
        let storage_error = self
            .api
            .persist_session(&self.session_id, &session)
            .err()
            .map(|error| error.to_string());
        self.api
            .set_game_state(snapshot.clone(), session.clone())
            .await;

        let settings = self.api.settings().await;
        let rendered = self
            .renderer
            .render(&snapshot, &session, settings.active_preset());

        match rendered {
            Ok(activity) => {
                let publish_error = self.presence.publish(&activity, now).err();
                let connected = publish_error.is_none();
                self.api.set_presence(Some(activity)).await;
                self.api
                    .set_status(RuntimeStatus {
                        telemetry_connected: true,
                        discord_connected: connected,
                        last_error: publish_error.or(storage_error),
                        ..RuntimeStatus::default()
                    })
                    .await;
            }
            Err(error) => {
                let _ = self.presence.clear();
                self.api.set_presence(None).await;
                self.api
                    .set_status(RuntimeStatus {
                        telemetry_connected: true,
                        discord_connected: false,
                        last_error: Some(error.to_string()),
                        ..RuntimeStatus::default()
                    })
                    .await;
            }
        }
    }

    async fn handle_unreachable(&mut self, error: String, now: DateTime<Utc>) {
        let mut snapshot = self
            .game
            .observe(Observation {
                reachable: false,
                map_valid: false,
                vehicle_name: None,
                vehicle_kind: Default::default(),
                map_generation: None,
            })
            .current;
        snapshot.telemetry = Telemetry::default();
        snapshot.captured_at = now;
        debug_assert_eq!(snapshot.phase, GamePhase::Offline);

        let session = self.session.apply(&snapshot, &[]);
        let storage_error = self
            .api
            .persist_session(&self.session_id, &session)
            .err()
            .map(|storage| storage.to_string());
        self.api.set_game_state(snapshot, session).await;
        let clear_error = self.presence.clear().err();
        self.api.set_presence(None).await;
        self.api
            .set_status(RuntimeStatus {
                telemetry_connected: false,
                discord_connected: false,
                last_error: Some(
                    clear_error
                        .or(storage_error)
                        .map_or(error.clone(), |secondary| format!("{error}; {secondary}")),
                ),
                ..RuntimeStatus::default()
            })
            .await;
    }
}

pub struct DiscordPresenceSink {
    client: DiscordIpcClient,
    connected: bool,
}

impl DiscordPresenceSink {
    pub fn canonical() -> Self {
        Self::new(DiscordIdentity::application_id())
    }

    fn new(application_id: impl AsRef<str>) -> Self {
        Self {
            client: DiscordIpcClient::new(application_id),
            connected: false,
        }
    }

    fn connect(&mut self) -> Result<(), String> {
        if !self.connected {
            self.client.connect().map_err(|error| error.to_string())?;
            self.connected = true;
        }
        Ok(())
    }
}

impl PresenceSink for DiscordPresenceSink {
    fn publish(&mut self, activity: &DiscordActivity) -> Result<(), String> {
        self.connect()?;
        let payload = discord_activity(activity);
        if let Err(error) = self.client.set_activity(payload.clone()) {
            self.connected = false;
            self.client
                .reconnect()
                .and_then(|_| self.client.set_activity(payload))
                .map_err(|retry_error| format!("{error}; reconnect failed: {retry_error}"))?;
            self.connected = true;
        }
        Ok(())
    }

    fn clear(&mut self) -> Result<(), String> {
        if !self.connected {
            return Ok(());
        }
        self.client.clear_activity().map_err(|error| {
            self.connected = false;
            error.to_string()
        })
    }
}

fn discord_activity(activity: &DiscordActivity) -> Activity<'static> {
    let mut payload = Activity::new();
    if let Some(details) = &activity.details {
        payload = payload.details(details.clone());
    }
    if let Some(state) = &activity.state {
        payload = payload.state(state.clone());
    }
    if activity.large_image.is_some() || activity.small_image.is_some() {
        let mut assets = Assets::new();
        if let Some(image) = &activity.large_image {
            assets = assets.large_image(image.clone());
        }
        if let Some(image) = &activity.small_image {
            assets = assets.small_image(image.clone());
        }
        payload = payload.assets(assets);
    }
    if let Some(started_at) = activity.started_at {
        payload = payload.timestamps(Timestamps::new().start(started_at));
    }
    payload
}
