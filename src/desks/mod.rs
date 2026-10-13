//! One module per desk from the original design pass's table (§2 of that
//! unpushed Erlang/OTP passport design doc).
//! Each owns its own command, its own event, and the logic that decides
//! between them — read one module to understand one capability
//! completely, never several.

pub mod assign_custodian;
pub mod capture_biometric_sample;
pub mod disclose_data;
pub mod grant_data_access;
pub mod initiate_passport;
pub mod record_health_observation;
pub mod register_identity_document;
pub mod revoke_data_access;
pub mod transfer_custodianship;
