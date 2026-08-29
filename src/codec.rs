//! The ONLY module in this crate that knows domain types get stored as
//! CBOR. `Claim`, `Grant`, `BiometricSample`, `PassportEvent` — none of
//! them have a `to_cbor`/`from_cbor` method of their own; they don't
//! know or care how they're persisted. This module is the sole adapter
//! between them and [`macula_rust_sdk::cbor::Value`], the same role
//! `store` plays for SQLite specifically.

use macula_rust_sdk::cbor::Value;
use uuid::Uuid;

use crate::biometric_sample::BiometricSample;
use crate::claim::Claim;
use crate::denial_reason::DenialReason;
use crate::desks::assign_custodian::CustodianAssignedV1;
use crate::desks::capture_biometric_sample::BiometricSampleCapturedV1;
use crate::desks::disclose_data::{DataAccessDeniedV1, DataDisclosedV1};
use crate::desks::grant_data_access::DataAccessGrantedV1;
use crate::desks::initiate_passport::PassportInitiatedV1;
use crate::desks::record_health_observation::HealthObservationRecordedV1;
use crate::desks::register_identity_document::IdentityDocumentRegisteredV1;
use crate::desks::revoke_data_access::DataAccessRevokedV1;
use crate::desks::transfer_custodianship::CustodianshipTransferredV1;
use crate::dossier::PassportEvent;
use crate::grant::Grant;
use crate::holder::HolderKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecError(pub String);

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "codec error: {}", self.0)
    }
}

impl std::error::Error for CodecError {}

// ---- Value field extractors (private: only this module needs them) ----

fn missing(key: &str) -> CodecError {
    CodecError(format!("missing field {key:?}"))
}

fn wrong_type(key: &str, expected: &str) -> CodecError {
    CodecError(format!("field {key:?} is not {expected}"))
}

fn text(v: &Value, key: &str) -> Result<String, CodecError> {
    match v.get(key) {
        Some(Value::Text(s)) => Ok(s.clone()),
        Some(_) => Err(wrong_type(key, "text")),
        None => Err(missing(key)),
    }
}

fn opt_text(v: &Value, key: &str) -> Result<Option<String>, CodecError> {
    match v.get(key) {
        Some(Value::Text(s)) => Ok(Some(s.clone())),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(wrong_type(key, "text or null")),
    }
}

fn int(v: &Value, key: &str) -> Result<i64, CodecError> {
    match v.get(key) {
        Some(Value::Int(i)) => i64::try_from(*i).map_err(|_| wrong_type(key, "i64-range int")),
        Some(_) => Err(wrong_type(key, "int")),
        None => Err(missing(key)),
    }
}

fn opt_int(v: &Value, key: &str) -> Result<Option<i64>, CodecError> {
    match v.get(key) {
        Some(Value::Int(i)) => {
            Ok(Some(i64::try_from(*i).map_err(|_| wrong_type(key, "i64-range int"))?))
        }
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(wrong_type(key, "int or null")),
    }
}

fn bytes(v: &Value, key: &str) -> Result<Vec<u8>, CodecError> {
    match v.get(key) {
        Some(Value::Bytes(b)) => Ok(b.clone()),
        Some(_) => Err(wrong_type(key, "bytes")),
        None => Err(missing(key)),
    }
}

fn opt_bytes(v: &Value, key: &str) -> Result<Option<Vec<u8>>, CodecError> {
    match v.get(key) {
        Some(Value::Bytes(b)) => Ok(Some(b.clone())),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(wrong_type(key, "bytes or null")),
    }
}

fn raw(v: &Value, key: &str) -> Result<Value, CodecError> {
    v.get(key).cloned().ok_or_else(|| missing(key))
}

fn uuid(v: &Value, key: &str) -> Result<Uuid, CodecError> {
    let b = bytes(v, key)?;
    Uuid::from_slice(&b).map_err(|e| CodecError(format!("field {key:?} is not a uuid: {e}")))
}

fn uuid_value(id: Uuid) -> Value {
    Value::Bytes(id.as_bytes().to_vec())
}

fn holder_kind_str(k: HolderKind) -> &'static str {
    match k {
        HolderKind::Human => "human",
        HolderKind::Animal => "animal",
    }
}

fn holder_kind_from_str(s: &str) -> Result<HolderKind, CodecError> {
    match s {
        "human" => Ok(HolderKind::Human),
        "animal" => Ok(HolderKind::Animal),
        other => Err(CodecError(format!("unknown holder_kind {other:?}"))),
    }
}

fn denial_reason_str(r: DenialReason) -> &'static str {
    match r {
        DenialReason::NoMatchingGrant => "no_matching_grant",
        DenialReason::GrantExpired => "grant_expired",
        DenialReason::GrantRevoked => "grant_revoked",
    }
}

fn denial_reason_from_str(s: &str) -> Result<DenialReason, CodecError> {
    match s {
        "no_matching_grant" => Ok(DenialReason::NoMatchingGrant),
        "grant_expired" => Ok(DenialReason::GrantExpired),
        "grant_revoked" => Ok(DenialReason::GrantRevoked),
        other => Err(CodecError(format!("unknown denial reason {other:?}"))),
    }
}

// ---- Value types ----

pub fn encode_claim(c: &Claim) -> Value {
    Value::Map(vec![])
        .with_field("claim_type", Value::text(&c.claim_type))
        .with_field("value", c.value.clone())
        .with_field("issuer", c.issuer.as_deref().map(Value::text).unwrap_or(Value::Null))
        .with_field("captured_at", Value::Int(c.captured_at as i128))
        .with_field(
            "expires_at",
            c.expires_at.map(|t| Value::Int(t as i128)).unwrap_or(Value::Null),
        )
}

pub fn decode_claim(v: &Value) -> Result<Claim, CodecError> {
    Ok(Claim {
        claim_type: text(v, "claim_type")?,
        value: raw(v, "value")?,
        issuer: opt_text(v, "issuer")?,
        captured_at: int(v, "captured_at")?,
        expires_at: opt_int(v, "expires_at")?,
    })
}

pub fn encode_biometric_sample(s: &BiometricSample) -> Value {
    Value::Map(vec![])
        .with_field("modality", Value::text(&s.modality))
        .with_field("template", Value::Bytes(s.template.clone()))
        .with_field("captured_at", Value::Int(s.captured_at as i128))
}

pub fn decode_biometric_sample(v: &Value) -> Result<BiometricSample, CodecError> {
    Ok(BiometricSample {
        modality: text(v, "modality")?,
        template: bytes(v, "template")?,
        captured_at: int(v, "captured_at")?,
    })
}

pub fn encode_grant(g: &Grant) -> Value {
    Value::Map(vec![])
        .with_field("id", uuid_value(g.id))
        .with_field("claim_type_prefix", Value::text(&g.claim_type_prefix))
        .with_field("requester", Value::Bytes(g.requester.clone()))
        .with_field("purpose", Value::text(&g.purpose))
        .with_field(
            "expires_at",
            g.expires_at.map(|t| Value::Int(t as i128)).unwrap_or(Value::Null),
        )
}

pub fn decode_grant(v: &Value) -> Result<Grant, CodecError> {
    Ok(Grant {
        id: uuid(v, "id")?,
        claim_type_prefix: text(v, "claim_type_prefix")?,
        requester: bytes(v, "requester")?,
        purpose: text(v, "purpose")?,
        expires_at: opt_int(v, "expires_at")?,
    })
}

// ---- Events ----

/// The stable string tag a `PassportEvent` round-trips through, and
/// what `store` uses for its `event_kind` column — a query/indexing
/// convenience unrelated to which storage engine is in play, but still
/// a codec-shaped concern (it's the CBOR `"event"` field's value too),
/// so it lives here rather than as a method on the enum itself.
pub fn event_kind(event: &PassportEvent) -> &'static str {
    match event {
        PassportEvent::PassportInitiatedV1(_) => "passport_initiated_v1",
        PassportEvent::CustodianAssignedV1(_) => "custodian_assigned_v1",
        PassportEvent::CustodianshipTransferredV1(_) => "custodianship_transferred_v1",
        PassportEvent::IdentityDocumentRegisteredV1(_) => "identity_document_registered_v1",
        PassportEvent::BiometricSampleCapturedV1(_) => "biometric_sample_captured_v1",
        PassportEvent::HealthObservationRecordedV1(_) => "health_observation_recorded_v1",
        PassportEvent::DataAccessGrantedV1(_) => "data_access_granted_v1",
        PassportEvent::DataAccessRevokedV1(_) => "data_access_revoked_v1",
        PassportEvent::DataDisclosedV1(_) => "data_disclosed_v1",
        PassportEvent::DataAccessDeniedV1(_) => "data_access_denied_v1",
    }
}

pub fn encode_event(event: &PassportEvent) -> Value {
    let map = Value::Map(vec![]).with_field("event", Value::text(event_kind(event)));
    match event {
        PassportEvent::PassportInitiatedV1(e) => map
            .with_field("holder_kind", Value::text(holder_kind_str(e.holder_kind)))
            .with_field("initiated_at", Value::Int(e.initiated_at as i128)),

        PassportEvent::CustodianAssignedV1(e) => map
            .with_field("custodian", Value::Bytes(e.custodian.clone()))
            .with_field("assigned_at", Value::Int(e.assigned_at as i128))
            .with_field("reason", e.reason.as_deref().map(Value::text).unwrap_or(Value::Null)),

        PassportEvent::CustodianshipTransferredV1(e) => map
            .with_field("from", Value::Bytes(e.from.clone()))
            .with_field(
                "to",
                e.to.as_ref().map(|t| Value::Bytes(t.clone())).unwrap_or(Value::Null),
            )
            .with_field("transferred_at", Value::Int(e.transferred_at as i128)),

        PassportEvent::IdentityDocumentRegisteredV1(e) => map
            .with_field("claim", encode_claim(&e.claim))
            .with_field("registered_by", Value::Bytes(e.registered_by.clone())),

        PassportEvent::BiometricSampleCapturedV1(e) => map
            .with_field("sample", encode_biometric_sample(&e.sample))
            .with_field("captured_by", Value::Bytes(e.captured_by.clone())),

        PassportEvent::HealthObservationRecordedV1(e) => map
            .with_field("claim", encode_claim(&e.claim))
            .with_field("recorded_by", Value::Bytes(e.recorded_by.clone())),

        PassportEvent::DataAccessGrantedV1(e) => map
            .with_field("grant", encode_grant(&e.grant))
            .with_field("granted_by", Value::Bytes(e.granted_by.clone())),

        PassportEvent::DataAccessRevokedV1(e) => map
            .with_field("grant_id", uuid_value(e.grant_id))
            .with_field("revoked_by", Value::Bytes(e.revoked_by.clone()))
            .with_field("revoked_at", Value::Int(e.revoked_at as i128)),

        PassportEvent::DataDisclosedV1(e) => map
            .with_field("requester", Value::Bytes(e.requester.clone()))
            .with_field("claim_type", Value::text(&e.claim_type))
            .with_field("via_grant", uuid_value(e.via_grant))
            .with_field("disclosed_at", Value::Int(e.disclosed_at as i128)),

        PassportEvent::DataAccessDeniedV1(e) => map
            .with_field("requester", Value::Bytes(e.requester.clone()))
            .with_field("claim_type", Value::text(&e.claim_type))
            .with_field("reason", Value::text(denial_reason_str(e.reason)))
            .with_field("denied_at", Value::Int(e.denied_at as i128)),
    }
}

pub fn decode_event(v: &Value) -> Result<PassportEvent, CodecError> {
    let kind = text(v, "event")?;
    Ok(match kind.as_str() {
        "passport_initiated_v1" => PassportEvent::PassportInitiatedV1(PassportInitiatedV1 {
            holder_kind: holder_kind_from_str(&text(v, "holder_kind")?)?,
            initiated_at: int(v, "initiated_at")?,
        }),
        "custodian_assigned_v1" => PassportEvent::CustodianAssignedV1(CustodianAssignedV1 {
            custodian: bytes(v, "custodian")?,
            assigned_at: int(v, "assigned_at")?,
            reason: opt_text(v, "reason")?,
        }),
        "custodianship_transferred_v1" => {
            PassportEvent::CustodianshipTransferredV1(CustodianshipTransferredV1 {
                from: bytes(v, "from")?,
                to: opt_bytes(v, "to")?,
                transferred_at: int(v, "transferred_at")?,
            })
        }
        "identity_document_registered_v1" => {
            PassportEvent::IdentityDocumentRegisteredV1(IdentityDocumentRegisteredV1 {
                claim: decode_claim(&raw(v, "claim")?)?,
                registered_by: bytes(v, "registered_by")?,
            })
        }
        "biometric_sample_captured_v1" => {
            PassportEvent::BiometricSampleCapturedV1(BiometricSampleCapturedV1 {
                sample: decode_biometric_sample(&raw(v, "sample")?)?,
                captured_by: bytes(v, "captured_by")?,
            })
        }
        "health_observation_recorded_v1" => {
            PassportEvent::HealthObservationRecordedV1(HealthObservationRecordedV1 {
                claim: decode_claim(&raw(v, "claim")?)?,
                recorded_by: bytes(v, "recorded_by")?,
            })
        }
        "data_access_granted_v1" => PassportEvent::DataAccessGrantedV1(DataAccessGrantedV1 {
            grant: decode_grant(&raw(v, "grant")?)?,
            granted_by: bytes(v, "granted_by")?,
        }),
        "data_access_revoked_v1" => PassportEvent::DataAccessRevokedV1(DataAccessRevokedV1 {
            grant_id: uuid(v, "grant_id")?,
            revoked_by: bytes(v, "revoked_by")?,
            revoked_at: int(v, "revoked_at")?,
        }),
        "data_disclosed_v1" => PassportEvent::DataDisclosedV1(DataDisclosedV1 {
            requester: bytes(v, "requester")?,
            claim_type: text(v, "claim_type")?,
            via_grant: uuid(v, "via_grant")?,
            disclosed_at: int(v, "disclosed_at")?,
        }),
        "data_access_denied_v1" => PassportEvent::DataAccessDeniedV1(DataAccessDeniedV1 {
            requester: bytes(v, "requester")?,
            claim_type: text(v, "claim_type")?,
            reason: denial_reason_from_str(&text(v, "reason")?)?,
            denied_at: int(v, "denied_at")?,
        }),
        other => return Err(CodecError(format!("unknown event kind {other:?}"))),
    })
}

#[cfg(test)]
mod tests {
    use crate::desks::assign_custodian::CustodianAssignedV1;
    use crate::desks::capture_biometric_sample::BiometricSampleCapturedV1;
    use crate::desks::disclose_data::{DataAccessDeniedV1, DataDisclosedV1};
    use crate::desks::grant_data_access::DataAccessGrantedV1;
    use crate::desks::initiate_passport::PassportInitiatedV1;
    use crate::desks::record_health_observation::HealthObservationRecordedV1;
    use crate::desks::register_identity_document::IdentityDocumentRegisteredV1;
    use crate::desks::revoke_data_access::DataAccessRevokedV1;
    use crate::desks::transfer_custodianship::CustodianshipTransferredV1;

    use super::*;

    fn roundtrip(event: PassportEvent) {
        let bytes = macula_rust_sdk::cbor::encode(&encode_event(&event)).expect("encode");
        let decoded_value = macula_rust_sdk::cbor::decode(&bytes).expect("decode");
        let decoded = decode_event(&decoded_value).expect("decode_event");
        assert_eq!(event, decoded, "round-trip mismatch for {}", event_kind(&event));
    }

    #[test]
    fn every_variant_round_trips_through_cbor() {
        let key = |b: u8| vec![b; 32];
        let claim = Claim {
            claim_type: "identity.passport.icao9303".to_string(),
            value: Value::text("P<BEL..."),
            issuer: Some("Kingdom of Belgium".to_string()),
            captured_at: 1000,
            expires_at: Some(2000),
        };
        let sample = BiometricSample {
            modality: "face".to_string(),
            template: vec![1, 2, 3, 4],
            captured_at: 1000,
        };
        let grant = Grant {
            id: Uuid::now_v7(),
            claim_type_prefix: "identity.*".to_string(),
            requester: key(2),
            purpose: "border check".to_string(),
            expires_at: Some(9999),
        };

        roundtrip(PassportEvent::PassportInitiatedV1(PassportInitiatedV1 {
            holder_kind: HolderKind::Human,
            initiated_at: 1,
        }));
        roundtrip(PassportEvent::PassportInitiatedV1(PassportInitiatedV1 {
            holder_kind: HolderKind::Animal,
            initiated_at: 1,
        }));
        roundtrip(PassportEvent::CustodianAssignedV1(CustodianAssignedV1 {
            custodian: key(1),
            assigned_at: 2,
            reason: Some("minor".to_string()),
        }));
        roundtrip(PassportEvent::CustodianAssignedV1(CustodianAssignedV1 {
            custodian: key(1),
            assigned_at: 2,
            reason: None,
        }));
        roundtrip(PassportEvent::CustodianshipTransferredV1(CustodianshipTransferredV1 {
            from: key(1),
            to: Some(key(3)),
            transferred_at: 3,
        }));
        roundtrip(PassportEvent::CustodianshipTransferredV1(CustodianshipTransferredV1 {
            from: key(1),
            to: None,
            transferred_at: 3,
        }));
        roundtrip(PassportEvent::IdentityDocumentRegisteredV1(IdentityDocumentRegisteredV1 {
            claim: claim.clone(),
            registered_by: key(1),
        }));
        roundtrip(PassportEvent::BiometricSampleCapturedV1(BiometricSampleCapturedV1 {
            sample: sample.clone(),
            captured_by: key(1),
        }));
        roundtrip(PassportEvent::HealthObservationRecordedV1(HealthObservationRecordedV1 {
            claim: claim.clone(),
            recorded_by: key(1),
        }));
        roundtrip(PassportEvent::DataAccessGrantedV1(DataAccessGrantedV1 {
            grant: grant.clone(),
            granted_by: key(1),
        }));
        roundtrip(PassportEvent::DataAccessRevokedV1(DataAccessRevokedV1 {
            grant_id: grant.id,
            revoked_by: key(1),
            revoked_at: 4,
        }));
        roundtrip(PassportEvent::DataDisclosedV1(DataDisclosedV1 {
            requester: key(2),
            claim_type: "identity.passport.number".to_string(),
            via_grant: grant.id,
            disclosed_at: 5,
        }));
        for reason in [
            DenialReason::NoMatchingGrant,
            DenialReason::GrantExpired,
            DenialReason::GrantRevoked,
        ] {
            roundtrip(PassportEvent::DataAccessDeniedV1(DataAccessDeniedV1 {
                requester: key(2),
                claim_type: "identity.passport.number".to_string(),
                reason,
                denied_at: 6,
            }));
        }
    }

    #[test]
    fn unknown_event_kind_is_rejected() {
        let v = Value::Map(vec![]).with_field("event", Value::text("not_a_real_event"));
        assert!(decode_event(&v).is_err());
    }
}
