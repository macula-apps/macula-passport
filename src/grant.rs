use uuid::Uuid;

/// A subject's standing permission for `requester` to receive claims
/// matching `claim_type_prefix` via `disclose_data`, until `expires_at`.
///
/// `claim_type_prefix` matches a [`crate::claim::Claim::claim_type`]
/// either exactly (`"identity.passport.number"`) or as a wildcard prefix
/// (`"health.*"`). There is no `claim_type_prefix` shape that can ever
/// match a [`crate::biometric::BiometricSample`] — biometric samples
/// don't have a `claim_type` at all, so a `Grant` structurally cannot
/// authorize disclosing one. See [`crate::biometric`] for why.
#[derive(Debug, Clone, PartialEq)]
pub struct Grant {
    pub id: Uuid,
    pub claim_type_prefix: String,
    /// The requester's mesh pubkey.
    pub requester: Vec<u8>,
    pub purpose: String,
    pub expires_at: Option<i64>,
}
