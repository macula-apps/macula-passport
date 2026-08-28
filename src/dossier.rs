use std::collections::HashSet;

use uuid::Uuid;

use crate::biometric::BiometricSample;
use crate::claim::Claim;
use crate::event::{PassportEvent, SubjectKey, SubjectKind};
use crate::grant::Grant;

/// Current state of a subject's dossier, rebuilt by replaying its event
/// stream — never persisted directly, always derived. Per the design
/// doc's boundary rule, `disclose_data`'s decision reads only this
/// derived state, never an external read model.
#[derive(Debug, Clone, Default)]
pub struct Dossier {
    /// `None` until `PassportInitiatedV1` has been replayed.
    pub subject_kind: Option<SubjectKind>,
    pub custodian: Option<SubjectKey>,
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
            PassportEvent::PassportInitiatedV1 { subject_kind, .. } => {
                self.subject_kind = Some(*subject_kind);
            }
            PassportEvent::CustodianAssignedV1 { custodian, .. } => {
                self.custodian = Some(custodian.clone());
            }
            PassportEvent::CustodianshipTransferredV1 { to, .. } => {
                self.custodian = to.clone();
            }
            PassportEvent::IdentityDocumentRegisteredV1 { claim, .. } => {
                self.claims.push(claim.clone());
            }
            PassportEvent::BiometricSampleCapturedV1 { sample, .. } => {
                self.biometric_samples.push(sample.clone());
            }
            PassportEvent::HealthObservationRecordedV1 { claim, .. } => {
                self.claims.push(claim.clone());
            }
            PassportEvent::DataAccessGrantedV1 { grant, .. } => {
                self.grants.push(grant.clone());
            }
            PassportEvent::DataAccessRevokedV1 { grant_id, .. } => {
                self.revoked_grant_ids.insert(*grant_id);
            }
            PassportEvent::DataDisclosedV1 { .. } | PassportEvent::DataAccessDeniedV1 { .. } => {
                // Audit-only outcomes; they don't change replayable state.
            }
        }
    }

    pub fn is_initiated(&self) -> bool {
        self.subject_kind.is_some()
    }

    /// Grants that haven't been revoked. Expiry is checked separately at
    /// decision time against "now" — "active" here means "not revoked",
    /// a fact of history; "expired" depends on when you ask.
    pub fn active_grants(&self) -> impl Iterator<Item = &Grant> {
        self.grants.iter().filter(|g| !self.revoked_grant_ids.contains(&g.id))
    }

    pub fn grant_by_id(&self, id: Uuid) -> Option<&Grant> {
        self.grants.iter().find(|g| g.id == id)
    }

    pub fn is_grant_revoked(&self, id: Uuid) -> bool {
        self.revoked_grant_ids.contains(&id)
    }

    /// The most recently captured, unexpired claim matching `claim_type`
    /// exactly — what `disclose_data` hands over once a grant authorizes
    /// it. A subject can register the same `claim_type` more than once
    /// (a renewed passport); the newest unexpired one wins.
    pub fn current_claim(&self, claim_type: &str, now: i64) -> Option<&Claim> {
        self.claims
            .iter()
            .filter(|c| c.claim_type == claim_type)
            .filter(|c| !matches!(c.expires_at, Some(t) if t <= now))
            .max_by_key(|c| c.captured_at)
    }
}
