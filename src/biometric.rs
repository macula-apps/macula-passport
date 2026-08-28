use macula_rust_sdk::cbor::Value;

use crate::wire::{self, CodecError};

/// A biometric sample — face, fingerprint, iris template.
///
/// Deliberately NOT a [`crate::claim::Claim`]. Per the 2026-08-28 design
/// decision, raw biometric template bytes must never be disclosable —
/// unlike a passport number, a leaked template can't be rotated. Making
/// this a distinct type rather than a `Claim` with a "non-disclosable"
/// flag enforces that by construction: `grant_data_access` and
/// `disclose_data` operate on `Claim`/`claim_type`, and a
/// `BiometricSample` has no `claim_type` for a grant to even target —
/// there is no code path from a grant to these bytes, not a runtime
/// check someone has to remember to apply.
///
/// The only sanctioned mesh-facing operation on this data is a match
/// verdict against a freshly-supplied probe sample (a live capture from
/// the verifying party's own device, in the interactive "can I see your
/// passport" moment) — never the template itself. That verification flow
/// (procedure shape, and how to ensure it requires the subject's genuine
/// in-the-moment awareness rather than a silent remote probe) is not yet
/// designed; the type only guarantees the template can't leak through
/// the disclosure path that exists today.
#[derive(Debug, Clone, PartialEq)]
pub struct BiometricSample {
    /// "face", "fingerprint", "iris" — open string for the same reason
    /// `Claim::claim_type` is, not an enum enumerating every modality.
    pub modality: String,
    pub template: Vec<u8>,
    pub captured_at: i64,
}

impl BiometricSample {
    pub fn to_cbor(&self) -> Value {
        Value::Map(vec![])
            .with_field("modality", Value::text(&self.modality))
            .with_field("template", Value::Bytes(self.template.clone()))
            .with_field("captured_at", Value::Int(self.captured_at as i128))
    }

    pub fn from_cbor(v: &Value) -> Result<Self, CodecError> {
        Ok(BiometricSample {
            modality: wire::text(v, "modality")?,
            template: wire::bytes(v, "template")?,
            captured_at: wire::int(v, "captured_at")?,
        })
    }
}
