# macula-passport

[![CI](https://img.shields.io/github/actions/workflow/status/macula-apps/macula-passport/ci.yml?branch=master&label=CI)](https://github.com/macula-apps/macula-passport/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg)](#license)
[![Android](https://img.shields.io/badge/android-8.0%2B-3DDC84?logo=android&logoColor=white)](https://developer.android.com)
[![GitHub Sponsors](https://img.shields.io/badge/GitHub%20Sponsors-support-ea4aaa.svg?logo=githubsponsors&logoColor=white)](https://github.com/sponsors/rgfaber)

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/macula-passport-full-dark.svg">
    <img src="assets/macula-passport-full-light.svg" alt="Macula Passport" width="320">
  </picture>
</p>

<p align="center">
  <strong>Sovereign identity, biometric and health data vault, on the holder's own phone</strong>
</p>

---

> **Status, 2026-10-13:** v0 built, first Android app — `macula-passport`
> (`src/`) and its UniFFI bindings (`macula-passport-ffi/`) are both built,
> tested and structured: one module per capability, domain types with zero
> knowledge of how they're persisted. `android/` is the first app on them: a
> holder starts their passport on their own Android phone and records an
> identity document, kept only on that phone. No iOS app yet. See "Built so
> far" and "Still open" below for the real, current state.

## What is this?

A sovereign identity, biometric, and health data vault — for a human or an
animal in someone's care — event-sourced **on the holder's own device**,
disclosed to other parties only through an explicit, per-request,
consent-gated exchange. Never broadcast, never stored anywhere but here.

**What it is not:** a service you sign into, a broker between you and a
verifier, or a backup. There is no copy of a passport but the holder's own, and
no queue of disclosures waiting for the device to come back online — raw
biometric templates never leave the phone at all (see "Built so far").

## Install

<p align="center">
  <img src="assets/macula-passport-qr.svg" alt="QR code for https://macula.io/passport.apk" width="260" height="260">
</p>

Scan that with the phone, or open <https://macula.io/passport.apk> on it:
both lead to the latest signed release apk.

1. Scan the QR code on [macula.io/use](https://macula.io/use), or open
   <https://github.com/macula-apps/macula-passport/releases/latest/download/macula-passport.apk>
   on the phone (Android 8 or later). No GitHub account is needed.
2. Open the downloaded `macula-passport.apk`. The first time, Android asks
   to allow installs from your browser; allow it once.
3. Open Macula Passport and tap "Start my passport".

Every release is signed with the same key, so a newer `macula-passport.apk`
installs over the one you have and keeps your passport. To be told about
new versions, add this repository to
[Obtainium](https://github.com/ImranR98/Obtainium), which tracks GitHub
releases. Each release's notes carry the apk's sha256 and its signing
certificate.

**Releasing** (maintainers): a `vMAJOR.MINOR.PATCH` tag runs
`.github/workflows/release.yml`, which builds the release apk for both ABIs,
signs it with the release key and publishes it as `macula-passport.apk` on
the GitHub release. The key is in two repository secrets only, made once
by its owner with `scripts/create_release_key.sh`, which also writes the
owner's encrypted backup. Lose the key and no phone can update the app
again.

## Prior art, and why a separate repo

**Prior art to read before extending any of this:**

- An earlier design pass, unpushed — a passport service on Erlang/OTP,
  local-only, never published — is the domain model this repo inherits:
  one event-sourced dossier per holder, custodian/guardian model for
  minors and animals, and `disclose_data` as a synchronous, consent-gated
  RPC rather than a queryable read model. (That design doc itself still
  says "subject" — this repo corrected it to `holder`, since that's what
  a passport itself calls the person it's about; the Erlang doc hasn't
  been touched.)
- [`macula-io/macula-rust`](https://github.com/macula-io/macula-rust) —
  the mesh client this app builds on, and the reference for the
  Rust-core-plus-UniFFI-bindings pattern this app follows to reach iOS and
  Android from one implementation.

**Why a separate repo, and why here:** a phone is a more credible "the
holder's own device" than a server the holder happens to operate — most
people carry one, few people run a service. That earlier pass's domain
design carries over; its Erlang/OTP CMD/PRJ/QRY department structure does
not — this is a single mobile-native core, not a division. `macula-apps`
holds native mobile/CLI-shaped apps; desktop-style ones live in their own
org.

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
`macula_rust::cbor::Value`; `store.rs` is the sole adapter to SQLite.
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
`macula-rust-ffi`'s relationship to `macula-rust` (a separate
crate, so the core stays UniFFI-free); `FfiValue` mirrors that crate's
own restricted CBOR-value mirror (`Null`/`Int`/`Bytes`/`Text`/`Float` — no
`List`/`Map` yet, same limitation, not independently reinvented).

**Device identity and the Android libraries.** `deviceMeshKey(identityDir)`
returns the public key a holder acts as on their phone (`actingAs` on every
claim desk). On the first call it creates a macula-rust identity on the
fleet's `pq_hybrid` profile, with the admission puzzle solved, as an
owner-only key file in a directory the app names inside its private storage.
Every later call loads that same key. `scripts/build_android_libs.sh` builds
`libmacula_passport_ffi.so` for `arm64-v8a` and `x86_64` (minSdk 26) with
cargo-ndk, then generates the Kotlin bindings from that same library. It
needs `ANDROID_NDK_HOME`, cargo-ndk and the two Rust Android targets.
`macula-passport-ffi/tests/first_claim_on_the_phone.rs` walks the app's
first slice: it gets the device key, initiates a passport, registers an
identity document, drops the passport and reopens it, and the claim is
replayed from the file.

**The Android app (`android/`).** Kotlin and Compose, minSdk 26, sliced by
desk: `initiate_passport` (start screen), `register_identity_document` (the
document form: kind, number, issuing country picked from a list, expiry from a
date picker; one claim `identity.<passport|id_card|driving_licence>.number`
with the number as text, the ISO 3166-1 alpha-2 country as issuer and expiry at
the end of its last valid day, UTC) and `review_dossier` (the claims replayed
from the file). `PassportOnThisPhone` keeps `passport.db`, the holder id and
the device key under the app's private files. Backup is off: no cloud backup
and no device transfer of any of it. Build: `scripts/build_android_libs.sh
android/app/src/main`, then `gradle testDebugUnitTest assembleDebug lintDebug`
in `android/` (Gradle 9.7.1, JDK 21). Walked on an Android 16 emulator: the
first start creates the device key, the document is recorded, and after the
app is killed and reopened the same key loads and the claim is replayed.

**Verification:** a full grant→use→deny→revoke integration test at both
layers (`tests/lifecycle.rs` for the core, `macula-passport-ffi/tests/lifecycle.rs`
through the actual `FfiPassport` object, since FFI-layer marshalling isn't
exercised by the core's own test), CBOR round-trip tests for every event
variant, and real `uniffi-bindgen generate` runs for both `--language
kotlin` and `--language swift` — confirming the full API surface actually
codegens with correct types, not just that the Rust side compiles. No
Swift toolchain is used yet; the Kotlin side is built and run by the
Android app above and its CI job. `cargo test --workspace` and `cargo
clippy --workspace --all-targets` both clean throughout.

## Still open

CI (`.github/workflows/ci.yml`, GitHub-hosted) runs on every push and pull
request: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test
--workspace`, and a bindings check that builds the FFI library and generates
the Kotlin and Swift bindings from it; the `apk` job builds the Android
libraries with cargo-ndk, runs the app's unit tests and lint, and checks both
ABIs' libraries are in the debug APK it uploads. No Swift consumer app exists
yet. Field-level schemas
beyond the claims-model shape itself (which jurisdictions' documents,
which biometric modalities, a health-observation vocabulary) are each
still a real design decision, not yet made.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option, matching `macula-rust`'s convention (standard for the
Rust crate ecosystem this app's core is built in).
