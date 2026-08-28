use macula_rust_sdk::cbor::Value;
use uuid::Uuid;

use crate::biometric::BiometricSample;
use crate::claim::Claim;
use crate::grant::Grant;
use crate::wire::{self, CodecError};

/// A subject's mesh pubkey. Distinct type from a requester's or
/// custodian's pubkey only in name — all are `Vec<u8>` mesh identities —
/// kept separate here so a call site reads as "the subject" rather than
/// an unlabeled byte vector.
pub type SubjectKey = Vec<u8>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectKind {
    Human,
    Animal,
}

impl SubjectKind {
    fn as_str(self) -> &'static str {
        match self {
            SubjectKind::Human => "human",
            SubjectKind::Animal => "animal",
        }
    }

    fn from_str(s: &str) -> Result<Self, CodecError> {
        match s {
            "human" => Ok(SubjectKind::Human),
            "animal" => Ok(SubjectKind::Animal),
            other => Err(CodecError(format!("unknown subject_kind {other:?}"))),
        }
    }
}

/// Why a `disclose_data` attempt was denied — see
/// [`PassportEvent::DataAccessDeniedV1`]. A refused attempt is recorded,
/// not silently dropped, per the design doc's §3; this enum is what makes
/// that record queryable later rather than a free-text reason nobody can
/// filter on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialReason {
    NoMatchingGrant,
    GrantExpired,
    GrantRevoked,
}

impl DenialReason {
    fn as_str(self) -> &'static str {
        match self {
            DenialReason::NoMatchingGrant => "no_matching_grant",
            DenialReason::GrantExpired => "grant_expired",
            DenialReason::GrantRevoked => "grant_revoked",
        }
    }

    fn from_str(s: &str) -> Result<Self, CodecError> {
        match s {
            "no_matching_grant" => Ok(DenialReason::NoMatchingGrant),
            "grant_expired" => Ok(DenialReason::GrantExpired),
            "grant_revoked" => Ok(DenialReason::GrantRevoked),
            other => Err(CodecError(format!("unknown denial reason {other:?}"))),
        }
    }
}

/// One slip in a subject's dossier. Variant names mirror the design
/// doc's desk table 1:1 (`plans/DESIGN_HECATE_PASSPORT.md` §2) —
/// `initiate_passport` → `PassportInitiatedV1`, etc.
///
/// This enum is the append-only log's payload shape. It does not yet
/// have a command/handler layer in front of it (the `maybe_*` functions
/// that would validate a command against replayed state before appending
/// one of these) — that's the next increment, not this one.
#[derive(Debug, Clone, PartialEq)]
pub enum PassportEvent {
    PassportInitiatedV1 {
        subject_kind: SubjectKind,
        initiated_at: i64,
    },
    CustodianAssignedV1 {
        custodian: SubjectKey,
        assigned_at: i64,
        reason: Option<String>,
    },
    /// `to: None` means the subject becomes self-sovereign (e.g. a minor
    /// reaching majority). `to: Some(_)` means custody moves to another
    /// custodian (e.g. a pet changing owners).
    CustodianshipTransferredV1 {
        from: SubjectKey,
        to: Option<SubjectKey>,
        transferred_at: i64,
    },
    IdentityDocumentRegisteredV1 {
        claim: Claim,
        registered_by: SubjectKey,
    },
    BiometricSampleCapturedV1 {
        sample: BiometricSample,
        captured_by: SubjectKey,
    },
    HealthObservationRecordedV1 {
        claim: Claim,
        recorded_by: SubjectKey,
    },
    DataAccessGrantedV1 {
        grant: Grant,
        granted_by: SubjectKey,
    },
    DataAccessRevokedV1 {
        grant_id: Uuid,
        revoked_by: SubjectKey,
        revoked_at: i64,
    },
    DataDisclosedV1 {
        requester: SubjectKey,
        claim_type: String,
        via_grant: Uuid,
        disclosed_at: i64,
    },
    DataAccessDeniedV1 {
        requester: SubjectKey,
        claim_type: String,
        reason: DenialReason,
        denied_at: i64,
    },
}

impl PassportEvent {
    /// The `event` tag this variant round-trips through — also what a
    /// `Store` row's `event_kind` column holds, so a specific event kind
    /// can be queried without decoding every row's full CBOR payload.
    pub fn kind(&self) -> &'static str {
        match self {
            PassportEvent::PassportInitiatedV1 { .. } => "passport_initiated_v1",
            PassportEvent::CustodianAssignedV1 { .. } => "custodian_assigned_v1",
            PassportEvent::CustodianshipTransferredV1 { .. } => "custodianship_transferred_v1",
            PassportEvent::IdentityDocumentRegisteredV1 { .. } => "identity_document_registered_v1",
            PassportEvent::BiometricSampleCapturedV1 { .. } => "biometric_sample_captured_v1",
            PassportEvent::HealthObservationRecordedV1 { .. } => "health_observation_recorded_v1",
            PassportEvent::DataAccessGrantedV1 { .. } => "data_access_granted_v1",
            PassportEvent::DataAccessRevokedV1 { .. } => "data_access_revoked_v1",
            PassportEvent::DataDisclosedV1 { .. } => "data_disclosed_v1",
            PassportEvent::DataAccessDeniedV1 { .. } => "data_access_denied_v1",
        }
    }

    pub fn to_cbor(&self) -> Value {
        let map = Value::Map(vec![]).with_field("event", Value::text(self.kind()));
        match self {
            PassportEvent::PassportInitiatedV1 {
                subject_kind,
                initiated_at,
            } => map
                .with_field("subject_kind", Value::text(subject_kind.as_str()))
                .with_field("initiated_at", Value::Int(*initiated_at as i128)),

            PassportEvent::CustodianAssignedV1 {
                custodian,
                assigned_at,
                reason,
            } => map
                .with_field("custodian", Value::Bytes(custodian.clone()))
                .with_field("assigned_at", Value::Int(*assigned_at as i128))
                .with_field(
                    "reason",
                    reason.as_deref().map(Value::text).unwrap_or(Value::Null),
                ),

            PassportEvent::CustodianshipTransferredV1 {
                from,
                to,
                transferred_at,
            } => map
                .with_field("from", Value::Bytes(from.clone()))
                .with_field(
                    "to",
                    to.as_ref().map(|t| Value::Bytes(t.clone())).unwrap_or(Value::Null),
                )
                .with_field("transferred_at", Value::Int(*transferred_at as i128)),

            PassportEvent::IdentityDocumentRegisteredV1 {
                claim,
                registered_by,
            } => map
                .with_field("claim", claim.to_cbor())
                .with_field("registered_by", Value::Bytes(registered_by.clone())),

            PassportEvent::BiometricSampleCapturedV1 { sample, captured_by } => map
                .with_field("sample", sample.to_cbor())
                .with_field("captured_by", Value::Bytes(captured_by.clone())),

            PassportEvent::HealthObservationRecordedV1 { claim, recorded_by } => map
                .with_field("claim", claim.to_cbor())
                .with_field("recorded_by", Value::Bytes(recorded_by.clone())),

            PassportEvent::DataAccessGrantedV1 { grant, granted_by } => map
                .with_field("grant", grant.to_cbor())
                .with_field("granted_by", Value::Bytes(granted_by.clone())),

            PassportEvent::DataAccessRevokedV1 {
                grant_id,
                revoked_by,
                revoked_at,
            } => map
                .with_field("grant_id", wire::uuid_value(*grant_id))
                .with_field("revoked_by", Value::Bytes(revoked_by.clone()))
                .with_field("revoked_at", Value::Int(*revoked_at as i128)),

            PassportEvent::DataDisclosedV1 {
                requester,
                claim_type,
                via_grant,
                disclosed_at,
            } => map
                .with_field("requester", Value::Bytes(requester.clone()))
                .with_field("claim_type", Value::text(claim_type))
                .with_field("via_grant", wire::uuid_value(*via_grant))
                .with_field("disclosed_at", Value::Int(*disclosed_at as i128)),

            PassportEvent::DataAccessDeniedV1 {
                requester,
                claim_type,
                reason,
                denied_at,
            } => map
                .with_field("requester", Value::Bytes(requester.clone()))
                .with_field("claim_type", Value::text(claim_type))
                .with_field("reason", Value::text(reason.as_str()))
                .with_field("denied_at", Value::Int(*denied_at as i128)),
        }
    }

    pub fn from_cbor(v: &Value) -> Result<Self, CodecError> {
        let kind = wire::text(v, "event")?;
        Ok(match kind.as_str() {
            "passport_initiated_v1" => PassportEvent::PassportInitiatedV1 {
                subject_kind: SubjectKind::from_str(&wire::text(v, "subject_kind")?)?,
                initiated_at: wire::int(v, "initiated_at")?,
            },
            "custodian_assigned_v1" => PassportEvent::CustodianAssignedV1 {
                custodian: wire::bytes(v, "custodian")?,
                assigned_at: wire::int(v, "assigned_at")?,
                reason: wire::opt_text(v, "reason")?,
            },
            "custodianship_transferred_v1" => PassportEvent::CustodianshipTransferredV1 {
                from: wire::bytes(v, "from")?,
                to: wire::opt_bytes(v, "to")?,
                transferred_at: wire::int(v, "transferred_at")?,
            },
            "identity_document_registered_v1" => PassportEvent::IdentityDocumentRegisteredV1 {
                claim: Claim::from_cbor(&wire::value(v, "claim")?)?,
                registered_by: wire::bytes(v, "registered_by")?,
            },
            "biometric_sample_captured_v1" => PassportEvent::BiometricSampleCapturedV1 {
                sample: BiometricSample::from_cbor(&wire::value(v, "sample")?)?,
                captured_by: wire::bytes(v, "captured_by")?,
            },
            "health_observation_recorded_v1" => PassportEvent::HealthObservationRecordedV1 {
                claim: Claim::from_cbor(&wire::value(v, "claim")?)?,
                recorded_by: wire::bytes(v, "recorded_by")?,
            },
            "data_access_granted_v1" => PassportEvent::DataAccessGrantedV1 {
                grant: Grant::from_cbor(&wire::value(v, "grant")?)?,
                granted_by: wire::bytes(v, "granted_by")?,
            },
            "data_access_revoked_v1" => PassportEvent::DataAccessRevokedV1 {
                grant_id: wire::uuid(v, "grant_id")?,
                revoked_by: wire::bytes(v, "revoked_by")?,
                revoked_at: wire::int(v, "revoked_at")?,
            },
            "data_disclosed_v1" => PassportEvent::DataDisclosedV1 {
                requester: wire::bytes(v, "requester")?,
                claim_type: wire::text(v, "claim_type")?,
                via_grant: wire::uuid(v, "via_grant")?,
                disclosed_at: wire::int(v, "disclosed_at")?,
            },
            "data_access_denied_v1" => PassportEvent::DataAccessDeniedV1 {
                requester: wire::bytes(v, "requester")?,
                claim_type: wire::text(v, "claim_type")?,
                reason: DenialReason::from_str(&wire::text(v, "reason")?)?,
                denied_at: wire::int(v, "denied_at")?,
            },
            other => return Err(CodecError(format!("unknown event kind {other:?}"))),
        })
    }
}

#[cfg(test)]
mod tests {
    use macula_rust_sdk::cbor;

    use super::*;
    use crate::biometric::BiometricSample;
    use crate::claim::Claim;
    use crate::grant::Grant;

    fn roundtrip(event: PassportEvent) {
        let bytes = cbor::encode(&event.to_cbor()).expect("encode");
        let decoded_value = cbor::decode(&bytes).expect("decode");
        let decoded = PassportEvent::from_cbor(&decoded_value).expect("from_cbor");
        assert_eq!(event, decoded, "round-trip mismatch for {}", event.kind());
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

        roundtrip(PassportEvent::PassportInitiatedV1 {
            subject_kind: SubjectKind::Human,
            initiated_at: 1,
        });
        roundtrip(PassportEvent::PassportInitiatedV1 {
            subject_kind: SubjectKind::Animal,
            initiated_at: 1,
        });
        roundtrip(PassportEvent::CustodianAssignedV1 {
            custodian: key(1),
            assigned_at: 2,
            reason: Some("minor".to_string()),
        });
        roundtrip(PassportEvent::CustodianAssignedV1 {
            custodian: key(1),
            assigned_at: 2,
            reason: None,
        });
        roundtrip(PassportEvent::CustodianshipTransferredV1 {
            from: key(1),
            to: Some(key(3)),
            transferred_at: 3,
        });
        roundtrip(PassportEvent::CustodianshipTransferredV1 {
            from: key(1),
            to: None,
            transferred_at: 3,
        });
        roundtrip(PassportEvent::IdentityDocumentRegisteredV1 {
            claim: claim.clone(),
            registered_by: key(1),
        });
        roundtrip(PassportEvent::BiometricSampleCapturedV1 {
            sample: sample.clone(),
            captured_by: key(1),
        });
        roundtrip(PassportEvent::HealthObservationRecordedV1 {
            claim: claim.clone(),
            recorded_by: key(1),
        });
        roundtrip(PassportEvent::DataAccessGrantedV1 {
            grant: grant.clone(),
            granted_by: key(1),
        });
        roundtrip(PassportEvent::DataAccessRevokedV1 {
            grant_id: grant.id,
            revoked_by: key(1),
            revoked_at: 4,
        });
        roundtrip(PassportEvent::DataDisclosedV1 {
            requester: key(2),
            claim_type: "identity.passport.number".to_string(),
            via_grant: grant.id,
            disclosed_at: 5,
        });
        for reason in [
            DenialReason::NoMatchingGrant,
            DenialReason::GrantExpired,
            DenialReason::GrantRevoked,
        ] {
            roundtrip(PassportEvent::DataAccessDeniedV1 {
                requester: key(2),
                claim_type: "identity.passport.number".to_string(),
                reason,
                denied_at: 6,
            });
        }
    }

    #[test]
    fn unknown_event_kind_is_rejected() {
        let v = Value::Map(vec![]).with_field("event", Value::text("not_a_real_event"));
        assert!(PassportEvent::from_cbor(&v).is_err());
    }
}
