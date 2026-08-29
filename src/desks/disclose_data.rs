//! Desk: `disclose_data` — the mesh-facing boundary, and the one desk
//! that isn't an `emit_{event}_to_mesh` emitter. A third party's mesh
//! RPC call *is* this command. The handler checks the dossier's own
//! accumulated grant/revoke slips, replayed — never an external read
//! model — so the same command against the same event history always
//! produces the same decision, with no dependency on projection lag.
//!
//! Unlike every other desk, [`decide_disclosure`] never rejects a
//! command — see its own doc.

use uuid::Uuid;

use crate::denial_reason::DenialReason;
use crate::dossier::Dossier;
use crate::mesh_key::MeshKey;

#[derive(Debug, Clone, PartialEq)]
pub struct DiscloseDataV1 {
    pub requester: MeshKey,
    pub claim_type: String,
    pub at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataDisclosedV1 {
    pub requester: MeshKey,
    pub claim_type: String,
    pub via_grant: Uuid,
    pub disclosed_at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataAccessDeniedV1 {
    pub requester: MeshKey,
    pub claim_type: String,
    pub reason: DenialReason,
    pub denied_at: i64,
}

pub enum DiscloseDataOutcome {
    Disclosed(DataDisclosedV1),
    Denied(DataAccessDeniedV1),
}

/// Never rejects: a request that doesn't match an active grant is a
/// *denied* outcome, not an invalid command — "a refused attempt is
/// itself worth auditing" (design doc §3), so it's recorded either way,
/// not silently dropped the way an error would be.
pub fn decide_disclosure(state: &Dossier, cmd: DiscloseDataV1) -> DiscloseDataOutcome {
    let DiscloseDataV1 { requester, claim_type, at } = cmd;

    let matching = state
        .active_grants()
        .filter(|g| g.requester == requester && g.covers(&claim_type))
        .max_by_key(|g| g.expires_at.unwrap_or(i64::MAX));

    match matching {
        Some(g) if !g.is_expired(at) => DiscloseDataOutcome::Disclosed(DataDisclosedV1 {
            requester,
            claim_type,
            via_grant: g.id,
            disclosed_at: at,
        }),
        Some(_) => DiscloseDataOutcome::Denied(DataAccessDeniedV1 {
            requester,
            claim_type,
            reason: DenialReason::GrantExpired,
            denied_at: at,
        }),
        None => {
            let ever_granted =
                state.grants.iter().any(|g| g.requester == requester && g.covers(&claim_type));
            let reason =
                if ever_granted { DenialReason::GrantRevoked } else { DenialReason::NoMatchingGrant };
            DiscloseDataOutcome::Denied(DataAccessDeniedV1 { requester, claim_type, reason, denied_at: at })
        }
    }
}
