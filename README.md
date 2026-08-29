# macula-passport

A sovereign identity, biometric, and health data vault — for a human or an
animal in someone's care — event-sourced **on the holder's own device**,
disclosed to other parties only through an explicit, per-request,
consent-gated exchange. Never broadcast, never stored anywhere but here.

## Status: v0 built, no mobile app yet

`macula-passport` (`src/`) and its UniFFI bindings (`macula-passport-ffi/`)
are both built, tested, and structured — one module per capability, domain types
with zero knowledge of how they're persisted. What's still missing: CI, and
an actual Kotlin/Swift app consuming these bindings. See "Built so far" and
"Still open" below for the real, current state.

**Prior art to read before extending any of this:**

- `hecate-services/hecate-passport` — the original design pass, as an
  Erlang/OTP service on `hecate-om`. Local-only for now, not yet pushed.
  Its `plans/DESIGN_HECATE_PASSPORT.md` is the domain model this repo
  inherits: one event-sourced dossier per holder, custodian/guardian model
  for minors and animals, and `disclose_data` as a synchronous,
  consent-gated RPC rather than a queryable read model. (That design doc
  itself still says "subject" — this repo corrected it to `holder`, since
  that's what a passport itself calls the person it's about; the Erlang
  doc hasn't been touched.)
- [`macula-io/macula-rust-sdk`](https://github.com/macula-io/macula-rust-sdk) —
  the mesh client this app builds on, and the reference for the
  Rust-core-plus-UniFFI-bindings pattern this app follows to reach iOS and
  Android from one implementation.

**Why a separate repo, and why here:** a phone is a more credible "the
holder's own device" than a server the holder happens to operate — most
people carry one, few people run a service. `hecate-passport`'s domain
design carries over; its Erlang/OTP CMD/PRJ/QRY department structure does
not — this is a single mobile-native core, not a division. `macula-apps`
holds native mobile/CLI-shaped apps; `hecate-apps` stays for desktop-style
ones.

## Structure — one module per capability, not per technical kind

```
src/
  holder.rs, mesh_key.rs, claim.rs, biometric_sample.rs,
  grant.rs, denial_reason.rs   — shared domain vocabulary, used by 2+ desks
  desks/                        — one file per desk: its own command,
    initiate_passport.rs           its own event, its own decision logic.
    assign_custodian.rs            Read one file, understand one capability
    transfer_custodianship.rs      completely — never several.
    register_identity_document.rs
    capture_biometric_sample.rs
    record_health_observation.rs
    grant_data_access.rs
    revoke_data_access.rs
    disclose_data.rs
  dossier.rs   — the aggregate: replay state, plus the two thin enums
                 (PassportEvent, Command) that let store/handler hold
                 "any event"/"any command" without knowing what's inside
  handler.rs   — routes a Command to its own desk's function; no logic
  error.rs     — DomainError, the small shared rejection-reason set
  codec.rs     — the ONLY module that knows this crate uses CBOR
  store.rs     — the ONLY module that knows it uses SQLite
```

Domain types never have `to_cbor`/`from_cbor` methods on themselves — a
holder's dossier has no business knowing what serialization format it's
stored in. `codec.rs` is the sole adapter between domain types and
`macula_rust_sdk::cbor::Value`; `store.rs` is the sole adapter to SQLite.
This mirrors the same domain/infrastructure and vertical-slicing rules
already applied throughout the Erlang side of this workspace — general
principles, not Erlang-specific ones, corrected here 2026-08-29 after an
initial pass organized the crate by technical kind (one file with every
command, one with every event, one with every handler) instead.

## Built so far

**Field-level schemas, as a claims model rather than one schema per
document type.** `claim_type` is an open dotted string
(`"identity.passport.icao9303"`, `"health.vaccination.covid19"`), not an
enum enumerating every jurisdiction's document format — that enumeration
is a mobile-UI concern (which fields to prompt for a given `claim_type`),
not a core-schema one.

**Biometric samples are structurally non-disclosable.** Per an explicit
decision, raw template bytes must never leave the device —
`BiometricSample` (`src/biometric_sample.rs`) is deliberately not a
`Claim`: it has no `claim_type`, so `Grant`/`disclose_data` have no path
to it at all. The only sanctioned mesh operation on it is a match verdict
against a live-captured probe, not disclosure of the template — that
verification flow (procedure shape, and how to require genuine
in-the-moment holder awareness rather than a silent remote probe) isn't
designed yet.

**`disclose_data` stays synchronous and interactive-only** — no
server-side queue for offline delivery. A disclosure check is inherently
a face-to-face act; if the device isn't reachable, nothing happened, so
there's nothing to audit. A genuinely non-interactive disclosure need
(e.g. async KYC without the holder present) is a different primitive
entirely — a verifiable credential signed once while online, checkable
independently later — and would be a separate capability, not a queue
bolted onto this one.

**Storage is SQLite via `rusqlite`, `bundled` feature.** Not a bundled
Rust KV engine, not a full ES framework crate (`cqrs-es`, `eventsourced`,
`disintegrate`, `evento` — all real and current as of Aug 2026, but
shaped for server deployments this app doesn't need). The actual
requirement is narrow: an append-only events table, replay to rebuild the
dossier's grant/revoke state, a couple of local projections for the UI —
a schema to hand-roll, not a reason to take on a framework. `bundled`
compiles SQLite's C source alongside the Rust code rather than linking
whatever SQLite version happens to ship on a given device — more reliable
across iOS/Android builds than depending on the OS copy being present and
ABI-stable at link time.

**Command/handler layer and SQLite persistence.** `Store` (`src/store.rs`)
is the append-only SQLite log; `Dossier` (`src/dossier.rs`) replays a
holder's stream into current state; `handler::handle` (`src/handler.rs`)
routes a `Command` to its own desk's `maybe_*` function, which turns it
into an event or rejects it with a `DomainError` before anything is
appended. `disclose_data` is the one exception: it never rejects, it
decides `DataDisclosedV1` vs `DataAccessDeniedV1` (with the specific
reason: no matching grant, expired, or revoked) against
`Dossier::active_grants`, matching the design doc's determinism rule —
reads only the dossier's own replayed state, nothing external.

Local commands (`initiate_passport` through `revoke_data_access`) check
state preconditions only, not caller authority — they only ever
originate from the app running locally on the holder's own device, which
is already the trust boundary. `disclose_data` alone crosses an actual
security boundary (a remote mesh party), which is why it alone is
decided against the dossier's grants rather than simply accepted.

**UniFFI bindings** (`macula-passport-ffi/`, a separate workspace member).
`FfiPassport` is the one exported object: one method per desk (`initiate`,
`registerIdentityDocument`, `grantDataAccess`, `discloseData`, ...), each
doing load→replay→`handler::handle`→append as a single call — the mobile
side never manually replays. All decision logic stays in `macula-passport`;
nothing in `macula-passport-ffi` decides anything, it only marshals `Command` in
and `PassportEvent`/`DomainError` out. Structure mirrors
`macula-rust-sdk-ffi`'s relationship to `macula-rust-sdk` (a separate
crate, so the core stays UniFFI-free); `FfiValue` mirrors that crate's
own restricted CBOR-value mirror (`Null`/`Int`/`Bytes`/`Text`/`Float` — no
`List`/`Map` yet, same limitation, not independently reinvented).

**Verification:** a full grant→use→deny→revoke integration test at both
layers (`tests/lifecycle.rs` for the core, `macula-passport-ffi/tests/lifecycle.rs`
through the actual `FfiPassport` object, since FFI-layer marshalling isn't
exercised by the core's own test), CBOR round-trip tests for every event
variant, and real `uniffi-bindgen generate` runs for both `--language
kotlin` and `--language swift` — confirming the full API surface actually
codegens with correct types, not just that the Rust side compiles. No
Kotlin/Swift/mobile toolchain exists in this dev environment (same
constraint `macula-rust-sdk-ffi` has), so this is the same verification
bar that repo's own CI settles for. `cargo test --workspace` and `cargo
clippy --workspace --all-targets` both clean throughout.

## Still open

No CI workflow yet (the sibling SDK repos all run a bindgen codegen smoke
test in GitHub Actions; this repo doesn't have Actions configured at all)
— flagging as the natural next piece, not built unprompted. No actual
Kotlin/Swift consumer app exists — the bindings are verified to generate
correctly, not run against a real mobile build. Field-level schemas
beyond the claims-model shape itself (which jurisdictions' documents,
which biometric modalities, a health-observation vocabulary) are each
still a real design decision, not yet made.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option, matching `macula-rust-sdk`'s convention (standard for the
Rust crate ecosystem this app's core is built in).
