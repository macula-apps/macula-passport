//! Subject-owned identity/biometric/health dossier, event-sourced on-device.
//!
//! `Store` (SQLite) persists a subject's [`event::PassportEvent`] stream;
//! `Dossier` replays it into current state; `handler::handle` turns a
//! [`command::Command`] into an event, or rejects it with a
//! [`error::DomainError`] before anything is appended. No UniFFI bindings
//! yet — see the repo README for what's settled and what's still open.

pub mod biometric;
pub mod claim;
pub mod command;
pub mod dossier;
pub mod error;
pub mod event;
pub mod grant;
pub mod handler;
pub mod store;
mod wire;
