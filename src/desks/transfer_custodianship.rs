//! Desk: `transfer_custodianship`. Moving custody — a minor reaching
//! majority (`to: None`, self-sovereignty), or a pet changing owners
//! (`to: Some(_)`). Requires an active custodian to transfer *from*;
//! the first-ever custodian is `assign_custodian` instead.

use crate::dossier::Dossier;
use crate::error::DomainError;
use crate::mesh_key::MeshKey;

#[derive(Debug, Clone, PartialEq)]
pub struct TransferCustodianshipV1 {
    pub to: Option<MeshKey>,
    pub at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustodianshipTransferredV1 {
    pub from: MeshKey,
    pub to: Option<MeshKey>,
    pub transferred_at: i64,
}

pub fn maybe_transfer_custodianship(
    state: &Dossier,
    cmd: TransferCustodianshipV1,
) -> Result<CustodianshipTransferredV1, DomainError> {
    let from = state.custodian.clone().ok_or(DomainError::NoActiveCustodian)?;
    Ok(CustodianshipTransferredV1 { from, to: cmd.to, transferred_at: cmd.at })
}
