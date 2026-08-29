//! SQLite-backed append-only event log — the only module besides
//! `codec` that knows how this crate's events are stored. One
//! `holder_id` per dossier; `seq` is the 0-based position of an event
//! within that holder's stream, the ordering `Dossier::replay` depends
//! on.

use std::path::Path;

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::codec::{self, CodecError};
use crate::dossier::PassportEvent;

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
        holder_id BLOB NOT NULL,
        seq INTEGER NOT NULL,
        event_kind TEXT NOT NULL,
        payload BLOB NOT NULL,
        PRIMARY KEY (holder_id, seq)
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

    /// Appends `event` as the next slip in `holder_id`'s stream. The
    /// caller is responsible for having validated the event against
    /// replayed state first (`Store` just persists — see
    /// `crate::handler` for the validate-then-append flow).
    pub fn append(&mut self, holder_id: Uuid, event: &PassportEvent) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        let next_seq: i64 = tx.query_row(
            "SELECT COALESCE(MAX(seq) + 1, 0) FROM events WHERE holder_id = ?1",
            params![holder_id.as_bytes().to_vec()],
            |row| row.get(0),
        )?;
        let payload = macula_rust_sdk::cbor::encode(&codec::encode_event(event))
            .map_err(|e| StoreError::Codec(CodecError(e.to_string())))?;
        tx.execute(
            "INSERT INTO events (holder_id, seq, event_kind, payload) VALUES (?1, ?2, ?3, ?4)",
            params![holder_id.as_bytes().to_vec(), next_seq, codec::event_kind(event), payload],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Replays `holder_id`'s full stream in `seq` order.
    pub fn load(&self, holder_id: Uuid) -> Result<Vec<PassportEvent>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM events WHERE holder_id = ?1 ORDER BY seq ASC")?;
        let rows = stmt.query_map(params![holder_id.as_bytes().to_vec()], |row| {
            row.get::<_, Vec<u8>>(0)
        })?;

        let mut events = Vec::new();
        for row in rows {
            let payload = row?;
            let value = macula_rust_sdk::cbor::decode(&payload)
                .map_err(|e| StoreError::Codec(CodecError(e.to_string())))?;
            events.push(codec::decode_event(&value)?);
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use crate::desks::assign_custodian::CustodianAssignedV1;
    use crate::desks::initiate_passport::PassportInitiatedV1;
    use crate::holder::HolderKind;

    use super::*;

    #[test]
    fn append_then_load_preserves_order_and_content() {
        let mut store = Store::open_in_memory().unwrap();
        let holder = Uuid::now_v7();

        store
            .append(
                holder,
                &PassportEvent::PassportInitiatedV1(PassportInitiatedV1 {
                    holder_kind: HolderKind::Human,
                    initiated_at: 1,
                }),
            )
            .unwrap();
        store
            .append(
                holder,
                &PassportEvent::CustodianAssignedV1(CustodianAssignedV1 {
                    custodian: vec![9; 32],
                    assigned_at: 2,
                    reason: None,
                }),
            )
            .unwrap();

        let loaded = store.load(holder).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(matches!(loaded[0], PassportEvent::PassportInitiatedV1(_)));
        assert!(matches!(loaded[1], PassportEvent::CustodianAssignedV1(_)));
    }

    #[test]
    fn streams_for_different_holders_stay_isolated() {
        let mut store = Store::open_in_memory().unwrap();
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();

        store
            .append(
                a,
                &PassportEvent::PassportInitiatedV1(PassportInitiatedV1 {
                    holder_kind: HolderKind::Human,
                    initiated_at: 1,
                }),
            )
            .unwrap();

        assert_eq!(store.load(a).unwrap().len(), 1);
        assert_eq!(store.load(b).unwrap().len(), 0);
    }
}
