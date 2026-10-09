//! Desk: `register_identity_document`. Passport/national-ID/birth-
//! certificate facts, registered as a [`crate::claim::Claim`].

use crate::claim::Claim;
use crate::dossier::Dossier;
use crate::error::DomainError;
use crate::mesh_key::MeshKey;

#[derive(Debug, Clone, PartialEq)]
pub struct RegisterIdentityDocumentV1 {
    pub claim: Claim,
    pub acting_as: MeshKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDocumentRegisteredV1 {
    pub claim: Claim,
    pub registered_by: MeshKey,
}

pub fn maybe_register_identity_document(
    state: &Dossier,
    cmd: RegisterIdentityDocumentV1,
) -> Result<IdentityDocumentRegisteredV1, DomainError> {
    if !state.is_initiated() {
        return Err(DomainError::NotYetInitiated);
    }
    Ok(IdentityDocumentRegisteredV1 {
        claim: cmd.claim,
        registered_by: cmd.acting_as,
    })
}
