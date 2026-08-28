use macula_rust_sdk::cbor::Value;

use crate::wire::{self, CodecError};

/// A disclosable fact about the subject — an identity document, a health
/// observation, or anything else added later under a new `claim_type`.
///
/// Deliberately NOT one Rust type per document kind. `claim_type` is an
/// open, dotted string (`"identity.passport.icao9303"`,
/// `"health.vaccination.covid19"`) rather than an enum enumerating every
/// jurisdiction's document format up front — that enumeration is a mobile
/// UI concern (which fields to prompt for a given `claim_type`), not a
/// core schema concern. The core only needs to know a claim's shape, not
/// its contents.
///
/// `value` reuses [`macula_rust_sdk::cbor::Value`] rather than a
/// parallel semi-structured type, since a disclosed claim's value is
/// exactly what ends up as a `disclose_data` RPC reply payload — no
/// re-encoding step at that boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub claim_type: String,
    pub value: Value,
    /// Issuing authority, clinic, etc. `None` for a self-attested claim.
    pub issuer: Option<String>,
    pub captured_at: i64,
    pub expires_at: Option<i64>,
}

impl Claim {
    pub fn to_cbor(&self) -> Value {
        Value::Map(vec![])
            .with_field("claim_type", Value::text(&self.claim_type))
            .with_field("value", self.value.clone())
            .with_field(
                "issuer",
                self.issuer.as_deref().map(Value::text).unwrap_or(Value::Null),
            )
            .with_field("captured_at", Value::Int(self.captured_at as i128))
            .with_field(
                "expires_at",
                self.expires_at.map(|t| Value::Int(t as i128)).unwrap_or(Value::Null),
            )
    }

    pub fn from_cbor(v: &Value) -> Result<Self, CodecError> {
        Ok(Claim {
            claim_type: wire::text(v, "claim_type")?,
            value: wire::value(v, "value")?,
            issuer: wire::opt_text(v, "issuer")?,
            captured_at: wire::int(v, "captured_at")?,
            expires_at: wire::opt_int(v, "expires_at")?,
        })
    }
}
