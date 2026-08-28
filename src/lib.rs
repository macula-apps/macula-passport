//! Subject-owned identity/biometric/health dossier, event-sourced on-device.
//!
//! Storage substrate is decided (SQLite via `rusqlite`, `bundled`
//! feature) and the event schema exists (see [`event::PassportEvent`]),
//! but there is no command/handler layer yet — nothing here validates a
//! command against replayed state and appends an event as a consequence.
//! SQLite persistence (append/replay) is also not wired yet. Both are the
//! next increment, not this one. See the repo README for what's settled
//! and what's still open.

pub mod biometric;
pub mod claim;
pub mod event;
pub mod grant;
