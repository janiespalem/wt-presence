use chrono::{Duration, TimeZone, Utc};
use tempfile::tempdir;
use wt_presence::{
    session::{SessionEngine, SessionSummary},
    storage::SessionRepository,
};

fn summary(offset_minutes: i64, kills: u32) -> SessionSummary {
    let started_at = Utc.timestamp_opt(1_700_000_000, 0).unwrap()
        + Duration::minutes(offset_minutes);
    let mut value = SessionEngine::new(started_at).summary();
    value.kills = kills;
    value
}

#[test]
fn initializes_the_current_database_schema() {
    let directory = tempdir().unwrap();
    let repository = SessionRepository::open(directory.path().join("history.sqlite3")).unwrap();

    assert_eq!(repository.schema_version().unwrap(), 1);
}

#[test]
fn upserts_a_session_without_creating_duplicates() {
    let repository = SessionRepository::open_in_memory().unwrap();
    repository.save("current", &summary(0, 2)).unwrap();
    repository.save("current", &summary(0, 7)).unwrap();

    let sessions = repository.recent(10).unwrap();

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "current");
    assert_eq!(sessions[0].summary.kills, 7);
}

#[test]
fn returns_recent_sessions_newest_first_with_a_limit() {
    let repository = SessionRepository::open_in_memory().unwrap();
    repository.save("old", &summary(0, 1)).unwrap();
    repository.save("middle", &summary(10, 2)).unwrap();
    repository.save("new", &summary(20, 3)).unwrap();

    let sessions = repository.recent(2).unwrap();

    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].id, "new");
    assert_eq!(sessions[1].id, "middle");
}
