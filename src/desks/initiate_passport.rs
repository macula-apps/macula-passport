//! Desk: `initiate_passport`. Birth of a dossier — holder enrolled,
//! self-sovereign or (via a later `assign_custodian`) custodial.

use crate::dossier::Dossier;
use crate::error::DomainError;
use crate::holder::HolderKind;

#[derive(Debug, Clone, PartialEq)]
pub struct InitiatePassportV1 {
    pub holder_kind: HolderKind,
    pub at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PassportInitiatedV1 {
    pub holder_kind: HolderKind,
    pub initiated_at: i64,
}

pub fn maybe_initiate_passport(
    state: &Dossier,
    cmd: InitiatePassportV1,
) -> Result<PassportInitiatedV1, DomainError> {
    if state.is_initiated() {
        return Err(DomainError::AlreadyInitiated);
    }
    Ok(PassportInitiatedV1 { holder_kind: cmd.holder_kind, initiated_at: cmd.at })
}
