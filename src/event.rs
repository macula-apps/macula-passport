use uuid::Uuid;

use crate::biometric::BiometricSample;
use crate::claim::Claim;
use crate::grant::Grant;

/// A subject's mesh pubkey. Distinct type from a requester's or
/// custodian's pubkey only in name — all are `Vec<u8>` mesh identities —
/// kept separate here so a call site reads as "the subject" rather than
/// an unlabeled byte vector.
pub type SubjectKey = Vec<u8>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectKind {
    Human,
    Animal,
}

/// Why a `disclose_data` attempt was denied — see
/// [`PassportEvent::DataAccessDeniedV1`]. A refused attempt is recorded,
/// not silently dropped, per the design doc's §3; this enum is what makes
/// that record queryable later rather than a free-text reason nobody can
/// filter on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialReason {
    NoMatchingGrant,
    GrantExpired,
    GrantRevoked,
}

/// One slip in a subject's dossier. Variant names mirror the design
/// doc's desk table 1:1 (`plans/DESIGN_HECATE_PASSPORT.md` §2) —
/// `initiate_passport` → `PassportInitiatedV1`, etc.
///
/// This enum is the append-only log's payload shape. It does not yet
/// have a command/handler layer in front of it (the `maybe_*` functions
/// that would validate a command against replayed state before appending
/// one of these) — that's the next increment, not this one.
#[derive(Debug, Clone, PartialEq)]
pub enum PassportEvent {
    PassportInitiatedV1 {
        subject_kind: SubjectKind,
        initiated_at: i64,
    },
    CustodianAssignedV1 {
        custodian: SubjectKey,
        assigned_at: i64,
        reason: Option<String>,
    },
    /// `to: None` means the subject becomes self-sovereign (e.g. a minor
    /// reaching majority). `to: Some(_)` means custody moves to another
    /// custodian (e.g. a pet changing owners).
    CustodianshipTransferredV1 {
        from: SubjectKey,
        to: Option<SubjectKey>,
        transferred_at: i64,
    },
    IdentityDocumentRegisteredV1 {
        claim: Claim,
        registered_by: SubjectKey,
    },
    BiometricSampleCapturedV1 {
        sample: BiometricSample,
        captured_by: SubjectKey,
    },
    HealthObservationRecordedV1 {
        claim: Claim,
        recorded_by: SubjectKey,
    },
    DataAccessGrantedV1 {
        grant: Grant,
        granted_by: SubjectKey,
    },
    DataAccessRevokedV1 {
        grant_id: Uuid,
        revoked_by: SubjectKey,
        revoked_at: i64,
    },
    DataDisclosedV1 {
        requester: SubjectKey,
        claim_type: String,
        via_grant: Uuid,
        disclosed_at: i64,
    },
    DataAccessDeniedV1 {
        requester: SubjectKey,
        claim_type: String,
        reason: DenialReason,
        denied_at: i64,
    },
}
