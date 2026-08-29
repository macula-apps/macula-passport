//! End-to-end: initiate a dossier, register a claim, grant/deny/revoke
//! disclosure — through the real `Store` + `Dossier::replay` +
//! `handler::handle` path, not just unit-level pieces in isolation.

use macula_rust_sdk::cbor::Value;
use uuid::Uuid;

use macula_passport::claim::Claim;
use macula_passport::denial_reason::DenialReason;
use macula_passport::desks::assign_custodian::AssignCustodianV1;
use macula_passport::desks::disclose_data::DiscloseDataV1;
use macula_passport::desks::grant_data_access::GrantDataAccessV1;
use macula_passport::desks::initiate_passport::InitiatePassportV1;
use macula_passport::desks::register_identity_document::RegisterIdentityDocumentV1;
use macula_passport::desks::revoke_data_access::RevokeDataAccessV1;
use macula_passport::dossier::{Command, Dossier, PassportEvent};
use macula_passport::handler;
use macula_passport::holder::HolderKind;
use macula_passport::store::Store;

fn key(b: u8) -> Vec<u8> {
    vec![b; 32]
}

fn apply(store: &mut Store, holder: Uuid, cmd: Command) -> PassportEvent {
    let state = Dossier::replay(&store.load(holder).unwrap());
    let event = handler::handle(&state, cmd).expect("command rejected");
    store.append(holder, &event).unwrap();
    event
}

#[test]
fn disclosure_lifecycle_grant_use_deny_revoke() {
    let mut store = Store::open_in_memory().unwrap();
    let holder = Uuid::now_v7();
    let holder_key = key(1);
    let requester = key(2);

    apply(
        &mut store,
        holder,
        Command::InitiatePassportV1(InitiatePassportV1 { holder_kind: HolderKind::Human, at: 100 }),
    );

    apply(
        &mut store,
        holder,
        Command::RegisterIdentityDocumentV1(RegisterIdentityDocumentV1 {
            claim: Claim {
                claim_type: "identity.passport.number".to_string(),
                value: Value::text("BE1234567"),
                issuer: Some("Kingdom of Belgium".to_string()),
                captured_at: 100,
                expires_at: None,
            },
            acting_as: holder_key.clone(),
        }),
    );

    // No grant yet: disclose_data must deny, not error, and must record why.
    let state = Dossier::replay(&store.load(holder).unwrap());
    let denied = handler::handle(
        &state,
        Command::DiscloseDataV1(DiscloseDataV1 {
            requester: requester.clone(),
            claim_type: "identity.passport.number".to_string(),
            at: 150,
        }),
    )
    .unwrap();
    assert!(matches!(
        denied,
        PassportEvent::DataAccessDeniedV1(ref e) if e.reason == DenialReason::NoMatchingGrant
    ));
    store.append(holder, &denied).unwrap();

    // Grant access, then the same request must succeed.
    let granted_event = apply(
        &mut store,
        holder,
        Command::GrantDataAccessV1(GrantDataAccessV1 {
            claim_type_prefix: "identity.*".to_string(),
            requester: requester.clone(),
            purpose: "border check".to_string(),
            expires_at: Some(1000),
            acting_as: holder_key.clone(),
        }),
    );
    let grant_id = match granted_event {
        PassportEvent::DataAccessGrantedV1(e) => e.grant.id,
        _ => unreachable!(),
    };

    let state = Dossier::replay(&store.load(holder).unwrap());
    let disclosed = handler::handle(
        &state,
        Command::DiscloseDataV1(DiscloseDataV1 {
            requester: requester.clone(),
            claim_type: "identity.passport.number".to_string(),
            at: 200,
        }),
    )
    .unwrap();
    match &disclosed {
        PassportEvent::DataDisclosedV1(e) => assert_eq!(e.via_grant, grant_id),
        other => panic!("expected DataDisclosedV1, got {other:?}"),
    }
    store.append(holder, &disclosed).unwrap();

    // The disclosed value itself matches what was registered.
    let disclosed_claim = state.current_claim("identity.passport.number", 200).unwrap();
    assert_eq!(disclosed_claim.value, Value::text("BE1234567"));

    // Revoke, then the same request must deny again, distinguishing
    // "revoked" from "never granted".
    apply(
        &mut store,
        holder,
        Command::RevokeDataAccessV1(RevokeDataAccessV1 {
            grant_id,
            acting_as: holder_key.clone(),
            at: 300,
        }),
    );
    let state = Dossier::replay(&store.load(holder).unwrap());
    let denied_again = handler::handle(
        &state,
        Command::DiscloseDataV1(DiscloseDataV1 {
            requester,
            claim_type: "identity.passport.number".to_string(),
            at: 350,
        }),
    )
    .unwrap();
    assert!(matches!(
        denied_again,
        PassportEvent::DataAccessDeniedV1(ref e) if e.reason == DenialReason::GrantRevoked
    ));
}

#[test]
fn expired_grant_denies_with_the_right_reason() {
    let mut store = Store::open_in_memory().unwrap();
    let holder = Uuid::now_v7();
    let holder_key = key(1);
    let requester = key(2);

    apply(
        &mut store,
        holder,
        Command::InitiatePassportV1(InitiatePassportV1 { holder_kind: HolderKind::Human, at: 0 }),
    );
    apply(
        &mut store,
        holder,
        Command::GrantDataAccessV1(GrantDataAccessV1 {
            claim_type_prefix: "identity.*".to_string(),
            requester: requester.clone(),
            purpose: "border check".to_string(),
            expires_at: Some(100),
            acting_as: holder_key,
        }),
    );

    let state = Dossier::replay(&store.load(holder).unwrap());
    let denied = handler::handle(
        &state,
        Command::DiscloseDataV1(DiscloseDataV1 {
            requester,
            claim_type: "identity.passport.number".to_string(),
            at: 200, // past the grant's expiry
        }),
    )
    .unwrap();
    assert!(matches!(
        denied,
        PassportEvent::DataAccessDeniedV1(ref e) if e.reason == DenialReason::GrantExpired
    ));
}

#[test]
fn cannot_initiate_twice_or_assign_a_second_custodian() {
    let mut store = Store::open_in_memory().unwrap();
    let holder = Uuid::now_v7();

    apply(
        &mut store,
        holder,
        Command::InitiatePassportV1(InitiatePassportV1 { holder_kind: HolderKind::Animal, at: 0 }),
    );

    let state = Dossier::replay(&store.load(holder).unwrap());
    assert!(handler::handle(
        &state,
        Command::InitiatePassportV1(InitiatePassportV1 { holder_kind: HolderKind::Animal, at: 1 })
    )
    .is_err());

    apply(
        &mut store,
        holder,
        Command::AssignCustodianV1(AssignCustodianV1 {
            custodian: key(9),
            reason: Some("owner".to_string()),
            at: 1,
        }),
    );
    let state = Dossier::replay(&store.load(holder).unwrap());
    assert!(handler::handle(
        &state,
        Command::AssignCustodianV1(AssignCustodianV1 { custodian: key(8), reason: None, at: 2 })
    )
    .is_err());
}
