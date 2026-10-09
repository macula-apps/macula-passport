//! Desk: `capture_biometric_sample`. Fingerprint/face/iris templates —
//! see [`crate::biometric_sample`] for why these are never disclosable.

use crate::biometric_sample::BiometricSample;
use crate::dossier::Dossier;
use crate::error::DomainError;
use crate::mesh_key::MeshKey;

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureBiometricSampleV1 {
    pub sample: BiometricSample,
    pub acting_as: MeshKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BiometricSampleCapturedV1 {
    pub sample: BiometricSample,
    pub captured_by: MeshKey,
}

pub fn maybe_capture_biometric_sample(
    state: &Dossier,
    cmd: CaptureBiometricSampleV1,
) -> Result<BiometricSampleCapturedV1, DomainError> {
    if !state.is_initiated() {
        return Err(DomainError::NotYetInitiated);
    }
    Ok(BiometricSampleCapturedV1 {
        sample: cmd.sample,
        captured_by: cmd.acting_as,
    })
}
