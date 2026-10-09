//! Desk: `grant_data_access`. Category + requester + purpose + expiry —
//! the permission [`crate::desks::disclose_data`] later decides against.

use uuid::Uuid;

use crate::dossier::Dossier;
use crate::error::DomainError;
use crate::grant::Grant;
use crate::mesh_key::MeshKey;

#[derive(Debug, Clone, PartialEq)]
pub struct GrantDataAccessV1 {
    pub claim_type_prefix: String,
    pub requester: MeshKey,
    pub purpose: String,
    pub expires_at: Option<i64>,
    pub acting_as: MeshKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataAccessGrantedV1 {
    pub grant: Grant,
    pub granted_by: MeshKey,
}

pub fn maybe_grant_data_access(
    state: &Dossier,
    cmd: GrantDataAccessV1,
) -> Result<DataAccessGrantedV1, DomainError> {
    if !state.is_initiated() {
        return Err(DomainError::NotYetInitiated);
    }
    let grant = Grant {
        id: Uuid::now_v7(),
        claim_type_prefix: cmd.claim_type_prefix,
        requester: cmd.requester,
        purpose: cmd.purpose,
        expires_at: cmd.expires_at,
    };
    Ok(DataAccessGrantedV1 {
        grant,
        granted_by: cmd.acting_as,
    })
}
