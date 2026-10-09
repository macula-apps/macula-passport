//! Desk: `revoke_data_access`. Blocks *future* disclosure only — cannot
//! claw back what already left via a prior `disclose_data`.

use uuid::Uuid;

use crate::dossier::Dossier;
use crate::error::DomainError;
use crate::mesh_key::MeshKey;

#[derive(Debug, Clone, PartialEq)]
pub struct RevokeDataAccessV1 {
    pub grant_id: Uuid,
    pub acting_as: MeshKey,
    pub at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataAccessRevokedV1 {
    pub grant_id: Uuid,
    pub revoked_by: MeshKey,
    pub revoked_at: i64,
}

pub fn maybe_revoke_data_access(
    state: &Dossier,
    cmd: RevokeDataAccessV1,
) -> Result<DataAccessRevokedV1, DomainError> {
    if state.grant_by_id(cmd.grant_id).is_none() {
        return Err(DomainError::GrantNotFound);
    }
    if state.is_grant_revoked(cmd.grant_id) {
        return Err(DomainError::GrantAlreadyRevoked);
    }
    Ok(DataAccessRevokedV1 {
        grant_id: cmd.grant_id,
        revoked_by: cmd.acting_as,
        revoked_at: cmd.at,
    })
}
