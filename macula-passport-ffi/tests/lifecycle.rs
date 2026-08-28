//! Same grant→use→deny→revoke shape as the core crate's own
//! `tests/lifecycle.rs`, but through `FfiPassport` — this exercises the
//! FFI layer's own marshalling (`to_uuid`, the apply/state helpers, the
//! post-disclosure claim lookup), which the core crate's tests don't
//! touch at all.

use macula_passport_ffi::{
    new_subject_id, FfiClaim, FfiDenialReason, FfiDisclosureOutcome, FfiPassport, FfiSubjectKind,
    FfiValue,
};

fn key(b: u8) -> Vec<u8> {
    vec![b; 32]
}

fn temp_db_path(name: &str) -> String {
    std::env::temp_dir()
        .join(format!("macula-passport-ffi-test-{name}-{}.sqlite", std::process::id()))
        .to_string_lossy()
        .to_string()
}

#[test]
fn disclosure_lifecycle_through_the_ffi_object() {
    let path = temp_db_path("lifecycle");
    let subject_id = new_subject_id();
    assert_eq!(subject_id.len(), 16);

    let passport = FfiPassport::open(path, subject_id).unwrap();
    let subject_key = key(1);
    let requester = key(2);

    assert!(!passport.is_initiated().unwrap());
    passport.initiate(FfiSubjectKind::Human, 100).unwrap();
    assert!(passport.is_initiated().unwrap());

    passport
        .register_identity_document(
            FfiClaim {
                claim_type: "identity.passport.number".to_string(),
                value: FfiValue::Text("BE1234567".to_string()),
                issuer: Some("Kingdom of Belgium".to_string()),
                captured_at: 100,
                expires_at: None,
            },
            subject_key.clone(),
        )
        .unwrap();
    assert_eq!(passport.list_claims().unwrap().len(), 1);

    // No grant yet: denied, not an error.
    let denied = passport
        .disclose_data(requester.clone(), "identity.passport.number".to_string(), 150)
        .unwrap();
    assert!(matches!(
        denied,
        FfiDisclosureOutcome::Denied { reason: FfiDenialReason::NoMatchingGrant }
    ));

    let grant_id = passport
        .grant_data_access(
            "identity.*".to_string(),
            requester.clone(),
            "border check".to_string(),
            Some(1000),
            subject_key.clone(),
        )
        .unwrap();
    assert_eq!(grant_id.len(), 16);
    assert_eq!(passport.list_active_grants().unwrap().len(), 1);

    let disclosed = passport
        .disclose_data(requester.clone(), "identity.passport.number".to_string(), 200)
        .unwrap();
    match disclosed {
        FfiDisclosureOutcome::Disclosed { claim } => {
            assert_eq!(claim.value, FfiValue::Text("BE1234567".to_string()));
        }
        other => panic!("expected Disclosed, got {other:?}"),
    }

    passport.revoke_data_access(grant_id, subject_key, 300).unwrap();
    assert_eq!(passport.list_active_grants().unwrap().len(), 0);

    let denied_again = passport
        .disclose_data(requester, "identity.passport.number".to_string(), 350)
        .unwrap();
    assert!(matches!(
        denied_again,
        FfiDisclosureOutcome::Denied { reason: FfiDenialReason::GrantRevoked }
    ));
}

#[test]
fn rejects_a_grant_id_of_the_wrong_length() {
    let path = temp_db_path("bad-grant-id");
    let passport = FfiPassport::open(path, new_subject_id()).unwrap();
    passport.initiate(FfiSubjectKind::Human, 0).unwrap();

    let err = passport.revoke_data_access(vec![1, 2, 3], key(1), 1).unwrap_err();
    assert!(matches!(err, macula_passport_ffi::FfiError::WrongByteLength { expected: 16, actual: 3 }));
}
