//! SQLite-backed append-only event log. One `subject_id` per dossier;
//! `seq` is the 0-based position of an event within that subject's
//! stream — the ordering `Dossier::replay` depends on.

use std::path::Path;

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::event::PassportEvent;
use crate::wire::CodecError;

#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Codec(CodecError),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "sqlite error: {e}"),
            StoreError::Codec(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Sqlite(e)
    }
}

impl From<CodecError> for StoreError {
    fn from(e: CodecError) -> Self {
        StoreError::Codec(e)
    }
}

pub struct Store {
    conn: Connection,
}

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS events (
        subject_id BLOB NOT NULL,
        seq INTEGER NOT NULL,
        event_kind TEXT NOT NULL,
        payload BLOB NOT NULL,
        PRIMARY KEY (subject_id, seq)
    );
";

impl Store {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Store { conn })
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Store { conn })
    }

    /// Appends `event` as the next slip in `subject_id`'s stream. The
    /// caller is responsible for having validated the event against
    /// replayed state first (`Store` just persists — see `crate::handler`
    /// for the validate-then-append flow).
    pub fn append(&mut self, subject_id: Uuid, event: &PassportEvent) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        let next_seq: i64 = tx.query_row(
            "SELECT COALESCE(MAX(seq) + 1, 0) FROM events WHERE subject_id = ?1",
            params![subject_id.as_bytes().to_vec()],
            |row| row.get(0),
        )?;
        let payload = macula_rust_sdk::cbor::encode(&event.to_cbor())
            .map_err(|e| StoreError::Codec(CodecError(e.to_string())))?;
        tx.execute(
            "INSERT INTO events (subject_id, seq, event_kind, payload) VALUES (?1, ?2, ?3, ?4)",
            params![subject_id.as_bytes().to_vec(), next_seq, event.kind(), payload],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Replays `subject_id`'s full stream in `seq` order.
    pub fn load(&self, subject_id: Uuid) -> Result<Vec<PassportEvent>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM events WHERE subject_id = ?1 ORDER BY seq ASC")?;
        let rows = stmt.query_map(params![subject_id.as_bytes().to_vec()], |row| {
            row.get::<_, Vec<u8>>(0)
        })?;

        let mut events = Vec::new();
        for row in rows {
            let payload = row?;
            let value = macula_rust_sdk::cbor::decode(&payload)
                .map_err(|e| StoreError::Codec(CodecError(e.to_string())))?;
            events.push(PassportEvent::from_cbor(&value)?);
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use crate::event::SubjectKind;

    use super::*;

    #[test]
    fn append_then_load_preserves_order_and_content() {
        let mut store = Store::open_in_memory().unwrap();
        let subject = Uuid::now_v7();

        store
            .append(
                subject,
                &PassportEvent::PassportInitiatedV1 {
                    subject_kind: SubjectKind::Human,
                    initiated_at: 1,
                },
            )
            .unwrap();
        store
            .append(
                subject,
                &PassportEvent::CustodianAssignedV1 {
                    custodian: vec![9; 32],
                    assigned_at: 2,
                    reason: None,
                },
            )
            .unwrap();

        let loaded = store.load(subject).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(matches!(loaded[0], PassportEvent::PassportInitiatedV1 { .. }));
        assert!(matches!(loaded[1], PassportEvent::CustodianAssignedV1 { .. }));
    }

    #[test]
    fn streams_for_different_subjects_stay_isolated() {
        let mut store = Store::open_in_memory().unwrap();
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();

        store
            .append(
                a,
                &PassportEvent::PassportInitiatedV1 {
                    subject_kind: SubjectKind::Human,
                    initiated_at: 1,
                },
            )
            .unwrap();

        assert_eq!(store.load(a).unwrap().len(), 1);
        assert_eq!(store.load(b).unwrap().len(), 0);
    }
}
