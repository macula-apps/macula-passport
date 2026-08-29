//! Passport-holder identity/biometric/health dossier, event-sourced
//! on-device.
//!
//! One module per business capability, not per technical kind — see
//! [`desks`] for the actual domain logic; a desk module is where
//! `initiate_passport`, `grant_data_access`, `disclose_data` and the
//! rest each own their own command, event, and decision, completely.
//! `holder`, `mesh_key`, `claim`, `biometric_sample`, `grant`, and
//! `denial_reason` are the shared vocabulary nouns more than one desk
//! uses. `dossier` is the aggregate: replay state, plus the two thin
//! enums (`PassportEvent`, `Command`) that let `store` and `handler`
//! hold "any event"/"any command" without needing to know what's inside
//! one.
//!
//! Domain types have zero knowledge of persistence or wire format —
//! [`codec`] is the only module that knows this crate uses CBOR,
//! [`store`] is the only module that knows it uses SQLite.

pub mod biometric_sample;
pub mod claim;
mod codec;
pub mod denial_reason;
pub mod desks;
pub mod dossier;
pub mod error;
pub mod grant;
pub mod handler;
pub mod holder;
pub mod mesh_key;
pub mod store;
