//! UniFFI (Kotlin/Swift) bindings for [`macula_passport`]. A thin
//! wrapper, not a reimplementation — every decision (`disclose_data`,
//! grant validity, custodian preconditions) happens in
//! `macula_passport::handler::handle`; nothing here decides anything, it
//! only marshals `Command` in and `PassportEvent`/`DomainError` out.
//!
//! Structure mirrors `macula-rust-sdk-ffi`'s relationship to
//! `macula-rust-sdk` (itself following `iroh-ffi`'s relationship to
//! `iroh`): a separate crate depending on `macula-passport`, so that
//! crate carries zero UniFFI dependency and stays just as usable from
//! plain Rust as it was before this crate existed. It also mirrors
//! `macula-passport`'s own module shape one level further: one file per
//! desk under `desks/`, each contributing its own `impl FfiPassport`
//! block for that capability — Rust allows a type's methods to be split
//! across multiple `impl` blocks in different files, and UniFFI's
//! `#[uniffi::export]` collects them crate-wide regardless of which
//! file declared them, so nothing forces this crate into one flat file.
//!
//! [`value::FfiValue`] mirrors `macula-rust-sdk-ffi`'s own `FfiValue` —
//! same `Null`/`Int`/`Bytes`/`Text`/`Float` variants, same missing
//! `List`/`Map` (recursive UniFFI enums, deferred there too — not
//! redefined independently here, just carrying the same limitation
//! since [`macula_passport::claim::Claim::value`] is the same
//! [`macula_rust_sdk::cbor::Value`] type). Not reusing that crate's type
//! directly: pulling in `macula-rust-sdk-ffi` here would drag in its
//! tokio/async-trait dependencies for zero benefit, since nothing in
//! this crate is async — every [`dossier::FfiPassport`] method is a
//! synchronous SQLite read/replay/decide/append, including
//! `disclose_data`, which only *decides* here; a mobile app wires the
//! actual inbound mesh CALL itself (via `macula-rust-sdk-ffi`'s own
//! `FfiCallHandler`) and calls into `disclose_data` from inside that
//! handler — this crate doesn't know the mesh exists.
//!
//! Generate bindings with the `uniffi-bindgen` binary this crate also
//! builds, e.g.:
//! ```text
//! cargo build -p macula-passport-ffi --release
//! cargo run -p macula-passport-ffi --bin uniffi-bindgen -- generate \
//!     --library target/release/libmacula_passport_ffi.so \
//!     --language kotlin --out-dir bindings/kotlin
//! ```

uniffi::setup_scaffolding!();

use uuid::Uuid;

pub mod biometric_sample;
pub mod claim;
pub mod denial_reason;
pub mod desks;
pub mod dossier;
pub mod grant;
pub mod holder;
pub mod value;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    /// A command was rejected before it became an event (see
    /// [`macula_passport::error::DomainError`]) — NOT what you get for a
    /// denied `disclose_data`, which is a successful
    /// `desks::disclose_data::FfiDisclosureOutcome::Denied`, not an
    /// error.
    #[error("rejected: {reason}")]
    Domain { reason: String },
    #[error("storage error: {reason}")]
    Store { reason: String },
    #[error("a value could not cross the FFI boundary: {reason}")]
    UnrepresentableValue { reason: String },
    #[error("expected exactly {expected} bytes, got {actual}")]
    WrongByteLength { expected: u32, actual: u32 },
}

impl From<macula_passport::error::DomainError> for FfiError {
    fn from(e: macula_passport::error::DomainError) -> Self {
        FfiError::Domain { reason: e.to_string() }
    }
}

impl From<macula_passport::store::StoreError> for FfiError {
    fn from(e: macula_passport::store::StoreError) -> Self {
        FfiError::Store { reason: e.to_string() }
    }
}

/// 16 raw bytes -> [`Uuid`], with both lengths reported on mismatch —
/// UniFFI has no fixed-size byte array type, so holder/grant ids cross
/// the boundary as `Vec<u8>` and get validated here.
pub(crate) fn to_uuid(bytes: Vec<u8>) -> Result<Uuid, FfiError> {
    let actual = bytes.len() as u32;
    Uuid::from_slice(&bytes).map_err(|_| FfiError::WrongByteLength { expected: 16, actual })
}

/// Generates a fresh holder id (a v7 UUID, same scheme `macula_passport`
/// uses for grant ids) — call once per new dossier and persist the
/// result locally (e.g. the platform keychain), then pass it to every
/// future [`dossier::FfiPassport::open`] for that holder.
#[uniffi::export]
pub fn new_holder_id() -> Vec<u8> {
    Uuid::now_v7().as_bytes().to_vec()
}
