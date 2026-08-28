//! Subject-owned identity/biometric/health dossier, event-sourced on-device.
//!
//! No domain code yet. Storage substrate is decided (SQLite via `rusqlite`,
//! `bundled` feature — see the repo README), but the event schema, the
//! dossier's command/event set, and the `disclose_data` RPC surface all
//! depend on field-level decisions (identity document types, biometric
//! template formats, health-observation vocabulary, consent-grant taxonomy)
//! that haven't been made yet. Nothing here is a placeholder for that work;
//! it doesn't exist until those decisions do.
