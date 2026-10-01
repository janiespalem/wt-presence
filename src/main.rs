#![cfg_attr(windows, windows_subsystem = "windows")]

use std::{env, net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::{Context, Result, bail};
use axum::{Router, http::HeaderValue};
use chrono::Utc;
use directories::ProjectDirs;
use tokio::{net::TcpListener, task::JoinHandle};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use wt_presence::{
    api::{ApiState, router as api_router},
    config::SettingsStore,
    runtime::{DisabledPresenceSink, DiscordPresenceSink, PresenceSink, RuntimeEngine},
    storage::SessionRepository,
    telemetry::WtTelemetryClient,
};

#[cfg(windows)]
mod windows_tray;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .compact()
        .init();

    let paths = AppPaths::discover()?;
    std::fs::create_dir_all(&paths.data_dir)
        .with_context(|| format!("create data directory {}", paths.data_dir.display()))?;

    let settings_store = SettingsStore::new(&paths.settings);
    let loaded = settings_store.load_or_create()?;
    if let Some(backup) = loaded.recovered_from {
        warn!(path = %backup.display(), "recovered from a corrupt settings file");
    }
    let settings = loaded.settings;
    let sessions = SessionRepository::open(&paths.sessions)?;
    let address = SocketAddr::from(([127, 0, 0, 1], settings.dashboard_port));
    let origin = format!("http://{address}");
    let token = Uuid::new_v4().to_string();
    let api = ApiState::new(
        settings.clone(),
        settings_store,
        sessions,
        token.clone(),
        origin.clone(),
    );

    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("bind local dashboard at {address}"))?;
    let web_root = discover_web_root()?;
    let app = application(api.clone(), &web_root);
    let runtime = spawn_runtime(api, &settings).await?;

    let dashboard_url = format!("{origin}/#token={token}");
    let (_exit_sender, exit_receiver) = tokio::sync::mpsc::unbounded_channel();
    #[cfg(windows)]
    let _tray = windows_tray::WindowsTray::start(dashboard_url.clone(), _exit_sender.clone())?;
    info!(url = %origin, web_root = %web_root.display(), "WT Presence is ready");
    if settings.open_dashboard_on_start {
        if let Err(error) = webbrowser::open(&dashboard_url) {
            warn!(%error, "could not open the dashboard automatically");
        }
    }

    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal(exit_receiver));
    if let Err(error) = server.await {
        error!(%error, "local dashboard server stopped unexpectedly");
    }
    runtime.abort();
    Ok(())
}

fn application(api: ApiState, web_root: &std::path::Path) -> Router {
    let index = web_root.join("index.html");
    let static_files = ServeDir::new(web_root).not_found_service(ServeFile::new(index));
    api_router(api)
        .fallback_service(static_files)
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; base-uri 'none'; frame-ancestors 'none'",
            ),
        ))
        .layer(TraceLayer::new_for_http())
}

async fn spawn_runtime(
    api: ApiState,
    settings: &wt_presence::config::AppSettings,
) -> Result<JoinHandle<()>> {
    let telemetry = WtTelemetryClient::new(settings.telemetry_url.clone())?;
    let presence: Box<dyn PresenceSink> = settings
        .discord_application_id
        .as_deref()
        .filter(|id| !id.trim().is_empty())
        .map(|id| Box::new(DiscordPresenceSink::new(id)) as Box<dyn PresenceSink>)
        .unwrap_or_else(|| Box::new(DisabledPresenceSink));
    let mut runtime = RuntimeEngine::new(telemetry, presence, api, Utc::now());

    Ok(tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            runtime.tick(Utc::now()).await;
        }
    }))
}

async fn shutdown_signal(mut exit_receiver: tokio::sync::mpsc::UnboundedReceiver<()>) {
    tokio::select! {
        result = tokio::signal::ctrl_c() => {
            if let Err(error) = result {
                error!(%error, "failed to install shutdown signal handler");
            }
        }
        _ = exit_receiver.recv() => {}
    }
}

fn discover_web_root() -> Result<PathBuf> {
    let candidates = [
        env::var_os("WT_PRESENCE_WEB_DIR").map(PathBuf::from),
        env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|parent| parent.join("web"))),
        env::current_dir().ok().map(|path| path.join("web/dist")),
    ];
    candidates
        .into_iter()
        .flatten()
        .find(|path| path.join("index.html").is_file())
        .ok_or_else(|| {
            anyhow::anyhow!("dashboard assets not found; build web/ or set WT_PRESENCE_WEB_DIR")
        })
}

struct AppPaths {
    data_dir: PathBuf,
    settings: PathBuf,
    sessions: PathBuf,
}

impl AppPaths {
    fn discover() -> Result<Self> {
        let Some(project) = ProjectDirs::from("net", "WT Presence", "WT Presence") else {
            bail!("operating system did not provide an application data directory");
        };
        let data_dir = project.data_local_dir().to_path_buf();
        Ok(Self {
            settings: data_dir.join("settings.json"),
            sessions: data_dir.join("sessions.sqlite3"),
            data_dir,
        })
    }
}
