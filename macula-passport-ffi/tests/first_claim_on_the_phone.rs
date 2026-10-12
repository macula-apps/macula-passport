//! The first slice of the Android app (#3), as the app drives it: the
//! device's own identity key, a passport started on it, one identity
//! document registered, and the claim still there after the app is killed
//! and reopened, replayed from the events in its own storage.

use macula_passport_ffi::claim::FfiClaim;
use macula_passport_ffi::device_identity::device_mesh_key;
use macula_passport_ffi::dossier::FfiPassport;
use macula_passport_ffi::holder::FfiHolderKind;
use macula_passport_ffi::new_holder_id;
use macula_passport_ffi::value::FfiValue;
use macula_rust::node_key::carried_key_well_formed;
use macula_rust::profile::Profile;

fn app_files_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "macula-passport-ffi-phone-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn identity_document(captured_at: i64) -> FfiClaim {
    FfiClaim {
        claim_type: "identity.passport.number".into(),
        value: FfiValue::Text("XX0000001".into()),
        issuer: Some("ZZ".into()),
        captured_at,
        expires_at: Some(captured_at + 10 * 365 * 24 * 3600),
    }
}

#[test]
fn the_device_keeps_one_identity_key_across_starts() {
    let dir = app_files_dir("identity");
    let identity_dir = dir.join("identity").to_string_lossy().to_string();

    let first = device_mesh_key(identity_dir.clone()).unwrap();
    let second = device_mesh_key(identity_dir).unwrap();

    assert_eq!(first, second);
    assert!(carried_key_well_formed(&first, Profile::PqHybrid));
}

#[test]
fn a_claim_survives_the_app_being_killed_and_reopened() {
    let dir = app_files_dir("reopen");
    let db = dir.join("passport.db").to_string_lossy().to_string();
    let identity_dir = dir.join("identity").to_string_lossy().to_string();
    let holder_id = new_holder_id();
    let acting_as = device_mesh_key(identity_dir).unwrap();

    {
        let passport = FfiPassport::open(db.clone(), holder_id.clone()).unwrap();
        passport
            .initiate(FfiHolderKind::Human, 1_760_000_000)
            .unwrap();
        passport
            .register_identity_document(identity_document(1_760_000_100), acting_as)
            .unwrap();
    }

    let reopened = FfiPassport::open(db, holder_id).unwrap();
    assert!(reopened.is_initiated().unwrap());
    let claims = reopened.list_claims().unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].claim_type, "identity.passport.number");
    assert_eq!(claims[0].issuer.as_deref(), Some("ZZ"));
}
