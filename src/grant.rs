use macula_rust_sdk::cbor::Value;
use uuid::Uuid;

use crate::wire::{self, CodecError};

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

impl Grant {
    pub fn to_cbor(&self) -> Value {
        Value::Map(vec![])
            .with_field("id", wire::uuid_value(self.id))
            .with_field("claim_type_prefix", Value::text(&self.claim_type_prefix))
            .with_field("requester", Value::Bytes(self.requester.clone()))
            .with_field("purpose", Value::text(&self.purpose))
            .with_field(
                "expires_at",
                self.expires_at.map(|t| Value::Int(t as i128)).unwrap_or(Value::Null),
            )
    }

    pub fn from_cbor(v: &Value) -> Result<Self, CodecError> {
        Ok(Grant {
            id: wire::uuid(v, "id")?,
            claim_type_prefix: wire::text(v, "claim_type_prefix")?,
            requester: wire::bytes(v, "requester")?,
            purpose: wire::text(v, "purpose")?,
            expires_at: wire::opt_int(v, "expires_at")?,
        })
    }

    /// Whether `claim_type` falls under this grant: exact match, or a
    /// `"prefix.*"` wildcard match on the leading segments.
    pub fn covers(&self, claim_type: &str) -> bool {
        match self.claim_type_prefix.strip_suffix(".*") {
            Some(prefix) => {
                claim_type == prefix || claim_type.starts_with(&format!("{prefix}."))
            }
            None => claim_type == self.claim_type_prefix,
        }
    }

    pub fn is_expired(&self, now: i64) -> bool {
        matches!(self.expires_at, Some(t) if t <= now)
    }
}
