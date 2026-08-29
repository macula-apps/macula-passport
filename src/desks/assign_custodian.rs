//! Desk: `assign_custodian`. A guardian/owner takes consent authority
//! over a holder who cannot hold it themselves. Requires no custodian
//! be active yet — moving custody from one custodian to another, or to
//! self-sovereignty, is `transfer_custodianship` instead.

use crate::dossier::Dossier;
use crate::error::DomainError;
use crate::mesh_key::MeshKey;

#[derive(Debug, Clone, PartialEq)]
pub struct AssignCustodianV1 {
    pub custodian: MeshKey,
    pub reason: Option<String>,
    pub at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustodianAssignedV1 {
    pub custodian: MeshKey,
    pub assigned_at: i64,
    pub reason: Option<String>,
}

pub fn maybe_assign_custodian(
    state: &Dossier,
    cmd: AssignCustodianV1,
) -> Result<CustodianAssignedV1, DomainError> {
    if !state.is_initiated() {
        return Err(DomainError::NotYetInitiated);
    }
    if state.custodian.is_some() {
        return Err(DomainError::CustodianAlreadyAssigned);
    }
    Ok(CustodianAssignedV1 { custodian: cmd.custodian, assigned_at: cmd.at, reason: cmd.reason })
}
