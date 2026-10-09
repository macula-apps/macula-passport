use macula_passport::desks::disclose_data::DiscloseDataV1;
use macula_passport::dossier::{Command, PassportEvent};

use crate::claim::FfiClaim;
use crate::denial_reason::FfiDenialReason;
use crate::dossier::FfiPassport;
use crate::FfiError;

#[derive(uniffi::Enum, Debug, Clone)]
pub enum FfiDisclosureOutcome {
    Disclosed { claim: FfiClaim },
    Denied { reason: FfiDenialReason },
}

#[uniffi::export]
impl FfiPassport {
    /// The mesh-facing decision. Unlike every other method on this
    /// type, this never returns `Err` for a denial — a request that
    /// doesn't match an active grant is a [`FfiDisclosureOutcome::Denied`],
    /// a successful outcome that gets recorded either way, not a
    /// rejected command. `Err` here means something structural broke
    /// (storage, codec), not that the answer was no.
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
                let claim = state
                    .current_claim(&claim_type, at)
                    .cloned()
                    .ok_or_else(|| FfiError::Store {
                        reason: "disclosed but no current claim found -- inconsistent state"
                            .to_string(),
                    })?;
                Ok(FfiDisclosureOutcome::Disclosed {
                    claim: FfiClaim::try_from(claim)?,
                })
            }
            PassportEvent::DataAccessDeniedV1(e) => Ok(FfiDisclosureOutcome::Denied {
                reason: e.reason.into(),
            }),
            _ => unreachable!(
                "apply(DiscloseDataV1) always returns DataDisclosedV1 or DataAccessDeniedV1"
            ),
        }
    }
}
