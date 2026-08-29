use uuid::Uuid;

use crate::mesh_key::MeshKey;

/// A holder's standing permission for `requester` to receive claims
/// matching `claim_type_prefix` via `disclose_data`, until `expires_at`.
///
/// `claim_type_prefix` matches a [`crate::claim::Claim::claim_type`]
/// either exactly (`"identity.passport.number"`) or as a wildcard prefix
/// (`"health.*"`). There is no `claim_type_prefix` shape that can ever
/// match a [`crate::biometric_sample::BiometricSample`] — biometric
/// samples don't have a `claim_type` at all, so a `Grant` structurally
/// cannot authorize disclosing one. See [`crate::biometric_sample`] for
/// why.
///
/// `covers`/`is_expired` are domain behavior — genuinely part of what a
/// `Grant` *is* — not serialization, so they stay here rather than
/// moving to `crate::codec`.
#[derive(Debug, Clone, PartialEq)]
pub struct Grant {
    pub id: Uuid,
    pub claim_type_prefix: String,
    pub requester: MeshKey,
    pub purpose: String,
    pub expires_at: Option<i64>,
}

impl Grant {
    /// Whether `claim_type` falls under this grant: exact match, or a
    /// `"prefix.*"` wildcard match on the leading segments.
    pub fn covers(&self, claim_type: &str) -> bool {
        match self.claim_type_prefix.strip_suffix(".*") {
            Some(prefix) => claim_type == prefix || claim_type.starts_with(&format!("{prefix}.")),
            None => claim_type == self.claim_type_prefix,
        }
    }

    pub fn is_expired(&self, now: i64) -> bool {
        matches!(self.expires_at, Some(t) if t <= now)
    }
}
