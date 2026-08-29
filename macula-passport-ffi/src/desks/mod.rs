//! One module per desk, mirroring `macula_passport::desks` structure —
//! each adds its own `impl FfiPassport` block for that capability's
//! method(s). Rust allows a type's methods to be split across multiple
//! `impl` blocks in different files, and UniFFI's `#[uniffi::export]`
//! collects them crate-wide regardless of which file declared them, so
//! nothing forces `FfiPassport` into one flat file. See
//! `macula_passport::desks` for the actual decision logic — nothing
//! here decides anything, it only marshals in and out.

pub mod assign_custodian;
pub mod capture_biometric_sample;
pub mod disclose_data;
pub mod grant_data_access;
pub mod initiate_passport;
pub mod record_health_observation;
pub mod register_identity_document;
pub mod revoke_data_access;
pub mod transfer_custodianship;
