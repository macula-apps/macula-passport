use uuid::Uuid;

use crate::biometric::BiometricSample;
use crate::claim::Claim;
use crate::event::{SubjectKey, SubjectKind};

/// A request to change a dossier's state. Distinct from
/// [`crate::event::PassportEvent`]: a command might be rejected (see
/// [`crate::error::DomainError`]); an event is a fact that already
/// happened. [`crate::handler::handle`] turns an accepted command into
/// exactly one event.
///
/// Every variant but `DiscloseData` carries `acting_as`: the subject's
/// own key, or their custodian's — whichever is using the app on this
/// device right now. This is recorded for audit (whose action was this),
/// not checked for authority: these commands only ever originate from
/// the app running locally on the subject's own device, which is
/// already the trust boundary. `DiscloseData` is the one command that
/// crosses an actual security boundary (a remote mesh party), which is
/// why it alone is decided against `Dossier::active_grants`, not simply
/// accepted.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    InitiatePassport {
        subject_kind: SubjectKind,
        at: i64,
    },
    AssignCustodian {
        custodian: SubjectKey,
        reason: Option<String>,
        at: i64,
    },
    TransferCustodianship {
        to: Option<SubjectKey>,
        at: i64,
    },
    RegisterIdentityDocument {
        claim: Claim,
        acting_as: SubjectKey,
    },
    CaptureBiometricSample {
        sample: BiometricSample,
        acting_as: SubjectKey,
    },
    RecordHealthObservation {
        claim: Claim,
        acting_as: SubjectKey,
    },
    GrantDataAccess {
        claim_type_prefix: String,
        requester: SubjectKey,
        purpose: String,
        expires_at: Option<i64>,
        acting_as: SubjectKey,
    },
    RevokeDataAccess {
        grant_id: Uuid,
        acting_as: SubjectKey,
        at: i64,
    },
    DiscloseData {
        requester: SubjectKey,
        claim_type: String,
        at: i64,
    },
}
