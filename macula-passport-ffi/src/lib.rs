//! UniFFI (Kotlin/Swift) bindings for [`macula_passport`]. A thin
//! wrapper, not a reimplementation — every decision (`disclose_data`,
//! grant validity, custodian preconditions) happens in
//! `macula_passport::handler::handle`; nothing here decides anything, it
//! only marshals `Command` in and `PassportEvent`/`DomainError` out.
//!
//! Structure mirrors `macula-rust-sdk-ffi`'s relationship to
//! `macula-rust-sdk` (itself following `iroh-ffi`'s relationship to
//! `iroh`): a separate crate depending on the core one, so the core
//! crate carries zero UniFFI dependency and stays just as usable from
//! plain Rust as it was before this crate existed.
//!
//! [`FfiValue`] mirrors `macula-rust-sdk-ffi`'s own `FfiValue` — same
//! `Null`/`Int`/`Bytes`/`Text`/`Float` variants, same missing
//! `List`/`Map` (recursive UniFFI enums, deferred there too — not
//! redefined independently here, just carrying the same limitation
//! since [`macula_passport::claim::Claim::value`] is the same
//! [`macula_rust_sdk::cbor::Value`] type). Not reusing that crate's type
//! directly: pulling in `macula-rust-sdk-ffi` here would drag in its
//! tokio/async-trait dependencies for zero benefit, since nothing in
//! this crate is async — every `FfiPassport` method is a synchronous
//! SQLite read/replay/decide/append, including `disclose_data`, which
//! only *decides* here; a mobile app wires the actual inbound mesh CALL
//! itself (via `macula-rust-sdk-ffi`'s own `FfiCallHandler`) and calls
//! into `disclose_data` from inside that handler — this crate doesn't
//! know the mesh exists.
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

use std::path::Path;
use std::sync::Mutex;

use macula_passport::biometric_sample::BiometricSample;
use macula_passport::claim::Claim;
use macula_passport::denial_reason::DenialReason;
use macula_passport::desks::assign_custodian::AssignCustodianV1;
use macula_passport::desks::capture_biometric_sample::CaptureBiometricSampleV1;
use macula_passport::desks::disclose_data::DiscloseDataV1;
use macula_passport::desks::grant_data_access::GrantDataAccessV1;
use macula_passport::desks::initiate_passport::InitiatePassportV1;
use macula_passport::desks::record_health_observation::RecordHealthObservationV1;
use macula_passport::desks::register_identity_document::RegisterIdentityDocumentV1;
use macula_passport::desks::revoke_data_access::RevokeDataAccessV1;
use macula_passport::desks::transfer_custodianship::TransferCustodianshipV1;
use macula_passport::dossier::{Command, Dossier, PassportEvent};
use macula_passport::grant::Grant;
use macula_passport::holder::HolderKind;
use macula_passport::store::Store;
use uuid::Uuid;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    /// A command was rejected before it became an event (see
    /// [`macula_passport::error::DomainError`]) — NOT what you get for a
    /// denied `disclose_data`, which is a successful
    /// [`FfiDisclosureOutcome::Denied`], not an error.
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
fn to_uuid(bytes: Vec<u8>) -> Result<Uuid, FfiError> {
    let actual = bytes.len() as u32;
    Uuid::from_slice(&bytes).map_err(|_| FfiError::WrongByteLength { expected: 16, actual })
}

/// A restricted mirror of [`macula_rust_sdk::cbor::Value`] — see this
/// crate's module doc for why `List`/`Map` are missing.
#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum FfiValue {
    Null,
    Int(i64),
    Bytes(Vec<u8>),
    Text(String),
    Float(f64),
}

impl From<FfiValue> for macula_rust_sdk::cbor::Value {
    fn from(v: FfiValue) -> Self {
        use macula_rust_sdk::cbor::Value;
        match v {
            FfiValue::Null => Value::Null,
            FfiValue::Int(n) => Value::Int(n as i128),
            FfiValue::Bytes(b) => Value::Bytes(b),
            FfiValue::Text(t) => Value::Text(t),
            FfiValue::Float(f) => Value::Float(f),
        }
    }
}

impl TryFrom<macula_rust_sdk::cbor::Value> for FfiValue {
    type Error = FfiError;

    fn try_from(v: macula_rust_sdk::cbor::Value) -> Result<Self, FfiError> {
        use macula_rust_sdk::cbor::Value;
        match v {
            Value::Null => Ok(FfiValue::Null),
            Value::Int(n) => i64::try_from(n).map(FfiValue::Int).map_err(|_| {
                FfiError::UnrepresentableValue { reason: format!("integer {n} is outside i64 range") }
            }),
            Value::Bytes(b) => Ok(FfiValue::Bytes(b)),
            Value::Text(t) => Ok(FfiValue::Text(t)),
            Value::Float(f) => Ok(FfiValue::Float(f)),
            Value::List(_) => Err(FfiError::UnrepresentableValue {
                reason: "list values are not yet supported across the FFI boundary".to_string(),
            }),
            Value::Map(_) => Err(FfiError::UnrepresentableValue {
                reason: "map values are not yet supported across the FFI boundary".to_string(),
            }),
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiHolderKind {
    Human,
    Animal,
}

impl From<FfiHolderKind> for HolderKind {
    fn from(k: FfiHolderKind) -> Self {
        match k {
            FfiHolderKind::Human => HolderKind::Human,
            FfiHolderKind::Animal => HolderKind::Animal,
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiDenialReason {
    NoMatchingGrant,
    GrantExpired,
    GrantRevoked,
}

impl From<DenialReason> for FfiDenialReason {
    fn from(r: DenialReason) -> Self {
        match r {
            DenialReason::NoMatchingGrant => FfiDenialReason::NoMatchingGrant,
            DenialReason::GrantExpired => FfiDenialReason::GrantExpired,
            DenialReason::GrantRevoked => FfiDenialReason::GrantRevoked,
        }
    }
}

#[derive(uniffi::Record, Debug, Clone)]
pub struct FfiClaim {
    pub claim_type: String,
    pub value: FfiValue,
    pub issuer: Option<String>,
    pub captured_at: i64,
    pub expires_at: Option<i64>,
}

impl From<FfiClaim> for Claim {
    fn from(c: FfiClaim) -> Self {
        Claim {
            claim_type: c.claim_type,
            value: c.value.into(),
            issuer: c.issuer,
            captured_at: c.captured_at,
            expires_at: c.expires_at,
        }
    }
}

impl TryFrom<Claim> for FfiClaim {
    type Error = FfiError;

    fn try_from(c: Claim) -> Result<Self, FfiError> {
        Ok(FfiClaim {
            claim_type: c.claim_type,
            value: FfiValue::try_from(c.value)?,
            issuer: c.issuer,
            captured_at: c.captured_at,
            expires_at: c.expires_at,
        })
    }
}

#[derive(uniffi::Record, Debug, Clone)]
pub struct FfiBiometricSample {
    pub modality: String,
    pub template: Vec<u8>,
    pub captured_at: i64,
}

impl From<FfiBiometricSample> for BiometricSample {
    fn from(s: FfiBiometricSample) -> Self {
        BiometricSample { modality: s.modality, template: s.template, captured_at: s.captured_at }
    }
}

impl From<BiometricSample> for FfiBiometricSample {
    fn from(s: BiometricSample) -> Self {
        FfiBiometricSample { modality: s.modality, template: s.template, captured_at: s.captured_at }
    }
}

/// A grant's `id` crosses as raw bytes, same as everywhere else in this
/// crate — but a caller can never *construct* one: the only way to
/// obtain a grant id is a prior [`FfiPassport::grant_data_access`]
/// return value, since [`Grant::id`] is always freshly minted inside
/// the core crate, never accepted as input.
#[derive(uniffi::Record, Debug, Clone)]
pub struct FfiGrant {
    pub id: Vec<u8>,
    pub claim_type_prefix: String,
    pub requester: Vec<u8>,
    pub purpose: String,
    pub expires_at: Option<i64>,
}

impl From<Grant> for FfiGrant {
    fn from(g: Grant) -> Self {
        FfiGrant {
            id: g.id.as_bytes().to_vec(),
            claim_type_prefix: g.claim_type_prefix,
            requester: g.requester,
            purpose: g.purpose,
            expires_at: g.expires_at,
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone)]
pub enum FfiDisclosureOutcome {
    Disclosed { claim: FfiClaim },
    Denied { reason: FfiDenialReason },
}

/// Generates a fresh holder id (a v7 UUID, same scheme the core crate
/// uses for grant ids) — call once per new dossier and persist the
/// result locally (e.g. the platform keychain), then pass it to every
/// future [`FfiPassport::open`] for that holder.
#[uniffi::export]
pub fn new_holder_id() -> Vec<u8> {
    Uuid::now_v7().as_bytes().to_vec()
}

/// One holder's dossier, backed by a SQLite file at `path`. Every
/// method here does load-replay-decide-append (or load-replay-read) as
/// one call — the mobile side never manually replays.
#[derive(uniffi::Object)]
pub struct FfiPassport {
    store: Mutex<Store>,
    holder_id: Uuid,
}

impl FfiPassport {
    fn apply(&self, cmd: Command) -> Result<PassportEvent, FfiError> {
        let mut store = self.store.lock().expect("store mutex poisoned");
        let events = store.load(self.holder_id)?;
        let state = Dossier::replay(&events);
        let event = macula_passport::handler::handle(&state, cmd)?;
        store.append(self.holder_id, &event)?;
        Ok(event)
    }

    fn state(&self) -> Result<Dossier, FfiError> {
        let store = self.store.lock().expect("store mutex poisoned");
        let events = store.load(self.holder_id)?;
        Ok(Dossier::replay(&events))
    }
}

#[uniffi::export]
impl FfiPassport {
    #[uniffi::constructor]
    pub fn open(path: String, holder_id: Vec<u8>) -> Result<Self, FfiError> {
        Ok(FfiPassport {
            store: Mutex::new(Store::open(Path::new(&path))?),
            holder_id: to_uuid(holder_id)?,
        })
    }

    pub fn initiate(&self, holder_kind: FfiHolderKind, at: i64) -> Result<(), FfiError> {
        self.apply(Command::InitiatePassportV1(InitiatePassportV1 {
            holder_kind: holder_kind.into(),
            at,
        }))?;
        Ok(())
    }

    pub fn assign_custodian(
        &self,
        custodian: Vec<u8>,
        reason: Option<String>,
        at: i64,
    ) -> Result<(), FfiError> {
        self.apply(Command::AssignCustodianV1(AssignCustodianV1 { custodian, reason, at }))?;
        Ok(())
    }

    pub fn transfer_custodianship(&self, to: Option<Vec<u8>>, at: i64) -> Result<(), FfiError> {
        self.apply(Command::TransferCustodianshipV1(TransferCustodianshipV1 { to, at }))?;
        Ok(())
    }

    pub fn register_identity_document(
        &self,
        claim: FfiClaim,
        acting_as: Vec<u8>,
    ) -> Result<(), FfiError> {
        self.apply(Command::RegisterIdentityDocumentV1(RegisterIdentityDocumentV1 {
            claim: claim.into(),
            acting_as,
        }))?;
        Ok(())
    }

    pub fn capture_biometric_sample(
        &self,
        sample: FfiBiometricSample,
        acting_as: Vec<u8>,
    ) -> Result<(), FfiError> {
        self.apply(Command::CaptureBiometricSampleV1(CaptureBiometricSampleV1 {
            sample: sample.into(),
            acting_as,
        }))?;
        Ok(())
    }

    pub fn record_health_observation(
        &self,
        claim: FfiClaim,
        acting_as: Vec<u8>,
    ) -> Result<(), FfiError> {
        self.apply(Command::RecordHealthObservationV1(RecordHealthObservationV1 {
            claim: claim.into(),
            acting_as,
        }))?;
        Ok(())
    }

    /// Returns the new grant's id (16 bytes) — pass it back to
    /// [`Self::revoke_data_access`] later.
    pub fn grant_data_access(
        &self,
        claim_type_prefix: String,
        requester: Vec<u8>,
        purpose: String,
        expires_at: Option<i64>,
        acting_as: Vec<u8>,
    ) -> Result<Vec<u8>, FfiError> {
        let event = self.apply(Command::GrantDataAccessV1(GrantDataAccessV1 {
            claim_type_prefix,
            requester,
            purpose,
            expires_at,
            acting_as,
        }))?;
        match event {
            PassportEvent::DataAccessGrantedV1(e) => Ok(e.grant.id.as_bytes().to_vec()),
            _ => unreachable!("apply(GrantDataAccessV1) always returns DataAccessGrantedV1"),
        }
    }

    pub fn revoke_data_access(
        &self,
        grant_id: Vec<u8>,
        acting_as: Vec<u8>,
        at: i64,
    ) -> Result<(), FfiError> {
        self.apply(Command::RevokeDataAccessV1(RevokeDataAccessV1 {
            grant_id: to_uuid(grant_id)?,
            acting_as,
            at,
        }))?;
        Ok(())
    }

    /// The mesh-facing decision. Unlike every other method here, this
    /// never returns `Err` for a denial — a request that doesn't match
    /// an active grant is a [`FfiDisclosureOutcome::Denied`], a
    /// successful outcome that gets recorded either way, not a rejected
    /// command. `Err` here means something structural broke (storage,
    /// codec), not that the answer was no.
    pub fn disclose_data(
        &self,
        requester: Vec<u8>,
        claim_type: String,
        at: i64,
    ) -> Result<FfiDisclosureOutcome, FfiError> {
        let event = self.apply(Command::DiscloseDataV1(DiscloseDataV1 {
            requester,
            claim_type: claim_type.clone(),
            at,
        }))?;
        match event {
            PassportEvent::DataDisclosedV1(_) => {
                let state = self.state()?;
                let claim = state.current_claim(&claim_type, at).cloned().ok_or_else(|| {
                    FfiError::Store {
                        reason: "disclosed but no current claim found -- inconsistent state"
                            .to_string(),
                    }
                })?;
                Ok(FfiDisclosureOutcome::Disclosed { claim: FfiClaim::try_from(claim)? })
            }
            PassportEvent::DataAccessDeniedV1(e) => {
                Ok(FfiDisclosureOutcome::Denied { reason: e.reason.into() })
            }
            _ => unreachable!(
                "apply(DiscloseDataV1) always returns DataDisclosedV1 or DataAccessDeniedV1"
            ),
        }
    }

    pub fn is_initiated(&self) -> Result<bool, FfiError> {
        Ok(self.state()?.is_initiated())
    }

    pub fn custodian(&self) -> Result<Option<Vec<u8>>, FfiError> {
        Ok(self.state()?.custodian)
    }

    pub fn list_claims(&self) -> Result<Vec<FfiClaim>, FfiError> {
        self.state()?.claims.into_iter().map(FfiClaim::try_from).collect()
    }

    pub fn list_biometric_samples(&self) -> Result<Vec<FfiBiometricSample>, FfiError> {
        Ok(self.state()?.biometric_samples.into_iter().map(FfiBiometricSample::from).collect())
    }

    pub fn list_active_grants(&self) -> Result<Vec<FfiGrant>, FfiError> {
        let state = self.state()?;
        Ok(state.active_grants().cloned().map(FfiGrant::from).collect())
    }
}
