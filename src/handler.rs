//! One `maybe_*` function per desk in the design doc's table
//! (`plans/DESIGN_HECATE_PASSPORT.md` §2 in `hecate-services/hecate-passport`),
//! each validating a [`Command`] against replayed [`Dossier`] state and
//! turning it into exactly one [`PassportEvent`] — or rejecting it with a
//! [`DomainError`] before anything is appended.

use uuid::Uuid;

use crate::command::Command;
use crate::dossier::Dossier;
use crate::error::DomainError;
use crate::event::{DenialReason, PassportEvent, SubjectKey};
use crate::grant::Grant;

pub fn handle(state: &Dossier, cmd: Command) -> Result<PassportEvent, DomainError> {
    match cmd {
        Command::InitiatePassport { subject_kind, at } => {
            maybe_initiate_passport(state, subject_kind, at)
        }
        Command::AssignCustodian { custodian, reason, at } => {
            maybe_assign_custodian(state, custodian, reason, at)
        }
        Command::TransferCustodianship { to, at } => maybe_transfer_custodianship(state, to, at),
        Command::RegisterIdentityDocument { claim, acting_as } => {
            maybe_register_identity_document(state, claim, acting_as)
        }
        Command::CaptureBiometricSample { sample, acting_as } => {
            maybe_capture_biometric_sample(state, sample, acting_as)
        }
        Command::RecordHealthObservation { claim, acting_as } => {
            maybe_record_health_observation(state, claim, acting_as)
        }
        Command::GrantDataAccess {
            claim_type_prefix,
            requester,
            purpose,
            expires_at,
            acting_as,
        } => maybe_grant_data_access(state, claim_type_prefix, requester, purpose, expires_at, acting_as),
        Command::RevokeDataAccess { grant_id, acting_as, at } => {
            maybe_revoke_data_access(state, grant_id, acting_as, at)
        }
        Command::DiscloseData { requester, claim_type, at } => {
            Ok(decide_disclosure(state, requester, claim_type, at))
        }
    }
}

fn maybe_initiate_passport(
    state: &Dossier,
    subject_kind: crate::event::SubjectKind,
    at: i64,
) -> Result<PassportEvent, DomainError> {
    if state.is_initiated() {
        return Err(DomainError::AlreadyInitiated);
    }
    Ok(PassportEvent::PassportInitiatedV1 { subject_kind, initiated_at: at })
}

fn maybe_assign_custodian(
    state: &Dossier,
    custodian: SubjectKey,
    reason: Option<String>,
    at: i64,
) -> Result<PassportEvent, DomainError> {
    if !state.is_initiated() {
        return Err(DomainError::NotYetInitiated);
    }
    if state.custodian.is_some() {
        return Err(DomainError::CustodianAlreadyAssigned);
    }
    Ok(PassportEvent::CustodianAssignedV1 { custodian, assigned_at: at, reason })
}

fn maybe_transfer_custodianship(
    state: &Dossier,
    to: Option<SubjectKey>,
    at: i64,
) -> Result<PassportEvent, DomainError> {
    let from = state.custodian.clone().ok_or(DomainError::NoActiveCustodian)?;
    Ok(PassportEvent::CustodianshipTransferredV1 { from, to, transferred_at: at })
}

fn maybe_register_identity_document(
    state: &Dossier,
    claim: crate::claim::Claim,
    acting_as: SubjectKey,
) -> Result<PassportEvent, DomainError> {
    if !state.is_initiated() {
        return Err(DomainError::NotYetInitiated);
    }
    Ok(PassportEvent::IdentityDocumentRegisteredV1 { claim, registered_by: acting_as })
}

fn maybe_capture_biometric_sample(
    state: &Dossier,
    sample: crate::biometric::BiometricSample,
    acting_as: SubjectKey,
) -> Result<PassportEvent, DomainError> {
    if !state.is_initiated() {
        return Err(DomainError::NotYetInitiated);
    }
    Ok(PassportEvent::BiometricSampleCapturedV1 { sample, captured_by: acting_as })
}

fn maybe_record_health_observation(
    state: &Dossier,
    claim: crate::claim::Claim,
    acting_as: SubjectKey,
) -> Result<PassportEvent, DomainError> {
    if !state.is_initiated() {
        return Err(DomainError::NotYetInitiated);
    }
    Ok(PassportEvent::HealthObservationRecordedV1 { claim, recorded_by: acting_as })
}

fn maybe_grant_data_access(
    state: &Dossier,
    claim_type_prefix: String,
    requester: SubjectKey,
    purpose: String,
    expires_at: Option<i64>,
    acting_as: SubjectKey,
) -> Result<PassportEvent, DomainError> {
    if !state.is_initiated() {
        return Err(DomainError::NotYetInitiated);
    }
    let grant = Grant {
        id: Uuid::now_v7(),
        claim_type_prefix,
        requester,
        purpose,
        expires_at,
    };
    Ok(PassportEvent::DataAccessGrantedV1 { grant, granted_by: acting_as })
}

fn maybe_revoke_data_access(
    state: &Dossier,
    grant_id: Uuid,
    acting_as: SubjectKey,
    at: i64,
) -> Result<PassportEvent, DomainError> {
    if state.grant_by_id(grant_id).is_none() {
        return Err(DomainError::GrantNotFound);
    }
    if state.is_grant_revoked(grant_id) {
        return Err(DomainError::GrantAlreadyRevoked);
    }
    Ok(PassportEvent::DataAccessRevokedV1 { grant_id, revoked_by: acting_as, revoked_at: at })
}

/// The mesh-facing decision (design doc §3). Unlike every other handler
/// here, this never rejects a command — a request that doesn't match an
/// active grant is a *denied* outcome, not an invalid one, and gets
/// recorded either way, per "a refused attempt is itself worth auditing."
fn decide_disclosure(
    state: &Dossier,
    requester: SubjectKey,
    claim_type: String,
    at: i64,
) -> PassportEvent {
    let matching = state
        .active_grants()
        .filter(|g| g.requester == requester && g.covers(&claim_type))
        .max_by_key(|g| g.expires_at.unwrap_or(i64::MAX));

    match matching {
        Some(g) if !g.is_expired(at) => PassportEvent::DataDisclosedV1 {
            requester,
            claim_type,
            via_grant: g.id,
            disclosed_at: at,
        },
        Some(_) => PassportEvent::DataAccessDeniedV1 {
            requester,
            claim_type,
            reason: DenialReason::GrantExpired,
            denied_at: at,
        },
        None => {
            let ever_granted = state
                .grants
                .iter()
                .any(|g| g.requester == requester && g.covers(&claim_type));
            let reason = if ever_granted {
                DenialReason::GrantRevoked
            } else {
                DenialReason::NoMatchingGrant
            };
            PassportEvent::DataAccessDeniedV1 { requester, claim_type, reason, denied_at: at }
        }
    }
}
