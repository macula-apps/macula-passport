use macula_passport::biometric_sample::BiometricSample;

#[derive(uniffi::Record, Debug, Clone)]
pub struct FfiBiometricSample {
    pub modality: String,
    pub template: Vec<u8>,
    pub captured_at: i64,
}

impl From<FfiBiometricSample> for BiometricSample {
    fn from(s: FfiBiometricSample) -> Self {
        BiometricSample { modality: s.modality, template: s.template, captured_at: s.captured_at }
    }
}

impl From<BiometricSample> for FfiBiometricSample {
    fn from(s: BiometricSample) -> Self {
        FfiBiometricSample { modality: s.modality, template: s.template, captured_at: s.captured_at }
    }
}
