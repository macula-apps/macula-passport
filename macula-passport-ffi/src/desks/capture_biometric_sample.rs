use macula_passport::desks::capture_biometric_sample::CaptureBiometricSampleV1;
use macula_passport::dossier::Command;

use crate::biometric_sample::FfiBiometricSample;
use crate::dossier::FfiPassport;
use crate::FfiError;

#[uniffi::export]
impl FfiPassport {
    pub fn capture_biometric_sample(
        &self,
        sample: FfiBiometricSample,
        acting_as: Vec<u8>,
    ) -> Result<(), FfiError> {
        self.apply(Command::CaptureBiometricSampleV1(CaptureBiometricSampleV1 {
            sample: sample.into(),
            acting_as,
        }))?;
        Ok(())
    }
}
