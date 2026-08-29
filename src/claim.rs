use macula_rust_sdk::cbor::Value;

/// A disclosable fact about the holder — an identity document, a health
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
/// re-encoding step at that boundary. This is a data-shape choice, not a
/// serialization concern: unlike the storage format, `crate::codec`
/// doesn't own this, it's what a `Claim` fundamentally *is*.
///
/// This type has no serialization methods of its own — see `crate::codec`
/// for how a `Claim` becomes bytes. A `Claim` doesn't know or care.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub claim_type: String,
    pub value: Value,
    /// Issuing authority, clinic, etc. `None` for a self-attested claim.
    pub issuer: Option<String>,
    pub captured_at: i64,
    pub expires_at: Option<i64>,
}
