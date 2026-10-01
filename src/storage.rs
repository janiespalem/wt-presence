use std::{
    path::Path,
    sync::{Mutex, MutexGuard},
};

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::session::SessionSummary;

const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StoredSession {
    pub id: String,
    pub summary: SessionSummary,
}

#[derive(Debug)]
pub struct SessionRepository {
    connection: Mutex<Connection>,
}

impl SessionRepository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let connection = Connection::open(path)?;
        Self::from_connection(connection)
    }

    pub fn open_in_memory() -> Result<Self, StorageError> {
        let connection = Connection::open_in_memory()?;
        Self::from_connection(connection)
    }

    pub fn schema_version(&self) -> Result<u32, StorageError> {
        let version = self
            .connection()?
            .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))?;
        Ok(version)
    }

    pub fn save(&self, id: &str, summary: &SessionSummary) -> Result<(), StorageError> {
        let payload = serde_json::to_string(summary)?;
        self.connection()?.execute(
            "INSERT INTO sessions (id, started_at, updated_at, payload_json)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                started_at = excluded.started_at,
                updated_at = excluded.updated_at,
                payload_json = excluded.payload_json",
            params![
                id,
                summary.started_at.timestamp(),
                summary.updated_at.timestamp(),
                payload
            ],
        )?;
        Ok(())
    }

    pub fn recent(&self, limit: usize) -> Result<Vec<StoredSession>, StorageError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, payload_json FROM sessions
             ORDER BY started_at DESC, id DESC
             LIMIT ?1",
        )?;
        let rows = statement.query_map([limit.min(i64::MAX as usize) as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut sessions = Vec::new();
        for row in rows {
            let (id, payload) = row?;
            sessions.push(StoredSession {
                id,
                summary: serde_json::from_str(&payload)?,
            });
        }
        Ok(sessions)
    }

    fn from_connection(connection: Connection) -> Result<Self, StorageError> {
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS sessions (
                 id TEXT PRIMARY KEY NOT NULL,
                 started_at INTEGER NOT NULL,
                 updated_at INTEGER NOT NULL,
                 payload_json TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS sessions_started_at_idx
                 ON sessions(started_at DESC);
             PRAGMA user_version = 1;",
        )?;
        let repository = Self {
            connection: Mutex::new(connection),
        };
        debug_assert_eq!(repository.schema_version()?, SCHEMA_VERSION);
        Ok(repository)
    }

    fn connection(&self) -> Result<MutexGuard<'_, Connection>, StorageError> {
        self.connection
            .lock()
            .map_err(|_| StorageError::LockPoisoned)
    }
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("session database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("session serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("session database lock was poisoned")]
    LockPoisoned,
}
