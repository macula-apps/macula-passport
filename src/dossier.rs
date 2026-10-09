use std::collections::HashSet;

use uuid::Uuid;

use crate::biometric_sample::BiometricSample;
use crate::claim::Claim;
use crate::desks::assign_custodian::{AssignCustodianV1, CustodianAssignedV1};
use crate::desks::capture_biometric_sample::{BiometricSampleCapturedV1, CaptureBiometricSampleV1};
use crate::desks::disclose_data::{DataAccessDeniedV1, DataDisclosedV1, DiscloseDataV1};
use crate::desks::grant_data_access::{DataAccessGrantedV1, GrantDataAccessV1};
use crate::desks::initiate_passport::{InitiatePassportV1, PassportInitiatedV1};
use crate::desks::record_health_observation::{
    HealthObservationRecordedV1, RecordHealthObservationV1,
};
use crate::desks::register_identity_document::{
    IdentityDocumentRegisteredV1, RegisterIdentityDocumentV1,
};
use crate::desks::revoke_data_access::{DataAccessRevokedV1, RevokeDataAccessV1};
use crate::desks::transfer_custodianship::{CustodianshipTransferredV1, TransferCustodianshipV1};
use crate::grant::Grant;
use crate::holder::HolderKind;
use crate::mesh_key::MeshKey;

/// One slip in a holder's dossier. Each variant wraps the event struct
/// its own desk module (`crate::desks::*`) defines — read that module,
/// not this enum, to see a capability's full shape. This enum's only
/// job is being the one common type `Store`/`Dossier::replay` need to
/// hold "any event" as — it carries no logic of its own.
#[derive(Debug, Clone, PartialEq)]
pub enum PassportEvent {
    PassportInitiatedV1(PassportInitiatedV1),
    CustodianAssignedV1(CustodianAssignedV1),
    CustodianshipTransferredV1(CustodianshipTransferredV1),
    IdentityDocumentRegisteredV1(IdentityDocumentRegisteredV1),
    BiometricSampleCapturedV1(BiometricSampleCapturedV1),
    HealthObservationRecordedV1(HealthObservationRecordedV1),
    DataAccessGrantedV1(DataAccessGrantedV1),
    DataAccessRevokedV1(DataAccessRevokedV1),
    DataDisclosedV1(DataDisclosedV1),
    DataAccessDeniedV1(DataAccessDeniedV1),
}

/// A request to change a dossier's state — see each desk module
/// (`crate::desks::*`) for its own command's fields. Distinct from
/// [`PassportEvent`]: a command might be rejected
/// (`crate::error::DomainError`); an event is a fact that already
/// happened. Same "thin aggregator, no logic" role as `PassportEvent`.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    InitiatePassportV1(InitiatePassportV1),
    AssignCustodianV1(AssignCustodianV1),
    TransferCustodianshipV1(TransferCustodianshipV1),
    RegisterIdentityDocumentV1(RegisterIdentityDocumentV1),
    CaptureBiometricSampleV1(CaptureBiometricSampleV1),
    RecordHealthObservationV1(RecordHealthObservationV1),
    GrantDataAccessV1(GrantDataAccessV1),
    RevokeDataAccessV1(RevokeDataAccessV1),
    DiscloseDataV1(DiscloseDataV1),
}

/// Current state of a holder's dossier, rebuilt by replaying its event
/// stream — never persisted directly, always derived. Per the design
/// doc's boundary rule, `disclose_data`'s decision reads only this
/// derived state, never an external read model.
#[derive(Debug, Clone, Default)]
pub struct Dossier {
    /// `None` until `PassportInitiatedV1` has been replayed.
    pub holder_kind: Option<HolderKind>,
    pub custodian: Option<MeshKey>,
    pub claims: Vec<Claim>,
    pub biometric_samples: Vec<BiometricSample>,
    pub grants: Vec<Grant>,
    revoked_grant_ids: HashSet<Uuid>,
}

impl Dossier {
    pub fn replay(events: &[PassportEvent]) -> Self {
        let mut d = Dossier::default();
        for e in events {
            d.apply(e);
        }
        d
    }

    pub fn apply(&mut self, event: &PassportEvent) {
        match event {
            PassportEvent::PassportInitiatedV1(e) => {
                self.holder_kind = Some(e.holder_kind);
            }
            PassportEvent::CustodianAssignedV1(e) => {
                self.custodian = Some(e.custodian.clone());
            }
            PassportEvent::CustodianshipTransferredV1(e) => {
                self.custodian = e.to.clone();
            }
            PassportEvent::IdentityDocumentRegisteredV1(e) => {
                self.claims.push(e.claim.clone());
            }
            PassportEvent::BiometricSampleCapturedV1(e) => {
                self.biometric_samples.push(e.sample.clone());
            }
            PassportEvent::HealthObservationRecordedV1(e) => {
                self.claims.push(e.claim.clone());
            }
            PassportEvent::DataAccessGrantedV1(e) => {
                self.grants.push(e.grant.clone());
            }
            PassportEvent::DataAccessRevokedV1(e) => {
                self.revoked_grant_ids.insert(e.grant_id);
            }
            PassportEvent::DataDisclosedV1(_) | PassportEvent::DataAccessDeniedV1(_) => {
                // Audit-only outcomes; they don't change replayable state.
            }
        }
    }

    pub fn is_initiated(&self) -> bool {
        self.holder_kind.is_some()
    }

    /// Grants that haven't been revoked. Expiry is checked separately at
    /// decision time against "now" — "active" here means "not revoked",
    /// a fact of history; "expired" depends on when you ask.
    pub fn active_grants(&self) -> impl Iterator<Item = &Grant> {
        self.grants
            .iter()
            .filter(|g| !self.revoked_grant_ids.contains(&g.id))
    }

    pub fn grant_by_id(&self, id: Uuid) -> Option<&Grant> {
        self.grants.iter().find(|g| g.id == id)
    }

    pub fn is_grant_revoked(&self, id: Uuid) -> bool {
        self.revoked_grant_ids.contains(&id)
    }

    /// The most recently captured, unexpired claim matching `claim_type`
    /// exactly — what `disclose_data` hands over once a grant authorizes
    /// it. A holder can register the same `claim_type` more than once
    /// (a renewed passport); the newest unexpired one wins.
    pub fn current_claim(&self, claim_type: &str, now: i64) -> Option<&Claim> {
        self.claims
            .iter()
            .filter(|c| c.claim_type == claim_type)
            .filter(|c| !matches!(c.expires_at, Some(t) if t <= now))
            .max_by_key(|c| c.captured_at)
    }
}
