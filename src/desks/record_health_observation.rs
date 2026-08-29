//! Desk: `record_health_observation`. Vaccination, condition,
//! prescription, vitals — registered as a [`crate::claim::Claim`], same
//! as an identity document.

use crate::claim::Claim;
use crate::dossier::Dossier;
use crate::error::DomainError;
use crate::mesh_key::MeshKey;

#[derive(Debug, Clone, PartialEq)]
pub struct RecordHealthObservationV1 {
    pub claim: Claim,
    pub acting_as: MeshKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthObservationRecordedV1 {
    pub claim: Claim,
    pub recorded_by: MeshKey,
}

pub fn maybe_record_health_observation(
    state: &Dossier,
    cmd: RecordHealthObservationV1,
) -> Result<HealthObservationRecordedV1, DomainError> {
    if !state.is_initiated() {
        return Err(DomainError::NotYetInitiated);
    }
    Ok(HealthObservationRecordedV1 { claim: cmd.claim, recorded_by: cmd.acting_as })
}
