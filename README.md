# macula-passport

A sovereign identity, biometric, and health data vault — for a human or an
animal in someone's care — event-sourced **on the subject's own device**,
disclosed to other parties only through an explicit, per-request,
consent-gated exchange. Never broadcast, never stored anywhere but here.

## Status: design, mostly not code

The Cargo workspace exists as plumbing only — a `core` crate wired to
`rusqlite` — no domain code, no schema, no UniFFI bindings yet. Those
still need real decisions (see "Still open" below), not placeholders
standing in for them.

**Prior art to read before writing any of that:**

- `hecate-services/hecate-passport` — the original design pass, as an
  Erlang/OTP service on `hecate-om`. Local-only for now, not yet pushed.
  Its `plans/DESIGN_HECATE_PASSPORT.md` is the domain model this repo inherits:
  one event-sourced dossier per subject, custodian/guardian model for
  minors and animals, and `disclose_data` as a synchronous, consent-gated
  RPC rather than a queryable read model.
- [`macula-io/macula-rust-sdk`](https://github.com/macula-io/macula-rust-sdk) —
  the mesh client this app builds on, and the reference for the
  Rust-core-plus-UniFFI-bindings pattern this app is expected to follow to
  reach iOS and Android from one implementation.

**Why a separate repo, and why here:** a phone is a more credible "the
subject's own device" than a server the subject happens to operate — most
people carry one, few people run a service. `hecate-passport`'s domain
design carries over; its Erlang/OTP CMD/PRJ/QRY department structure does
not — this is a single mobile-native core, not a division. `macula-apps`
holds native mobile/CLI-shaped apps; `hecate-apps` stays for desktop-style
ones.

**Settled already, carried over from the design discussion:**
`disclose_data` stays synchronous and interactive-only — no server-side
queue for offline delivery. A disclosure check is inherently a
face-to-face act; if the device isn't reachable, nothing happened, so
there's nothing to audit. A genuinely non-interactive disclosure need
(e.g. async KYC without the subject present) is a different primitive
entirely — a verifiable credential signed once while online, checkable
independently later — and would be a separate capability, not a queue
bolted onto this one.

**Settled, 2026-08-28: storage is SQLite via `rusqlite`, `bundled` feature.**
Not a bundled Rust KV engine, not a full ES framework crate (`cqrs-es`,
`eventsourced`, `disintegrate`, `evento` — all real and current as of
Aug 2026, but shaped for server deployments this app doesn't need). The
actual requirement is narrow: an append-only events table, replay to
rebuild the dossier's grant/revoke state, a couple of local projections
for the UI — a schema to hand-roll, not a reason to take on a framework.
`bundled` compiles SQLite's C source alongside the Rust code rather than
linking whatever SQLite version happens to ship on a given device — more
reliable across iOS/Android builds than depending on the OS copy being
present and ABI-stable at link time, and still far lighter than the
alternatives above.

**Settled, 2026-08-28: field-level schemas, as a claims model rather than
one schema per document type.** `claim_type` is an open dotted string
(`"identity.passport.icao9303"`, `"health.vaccination.covid19"`), not an
enum enumerating every jurisdiction's document format — that enumeration
is a mobile-UI concern (which fields to prompt for a given `claim_type`),
not a core-schema one. See `src/claim.rs`, `src/grant.rs`, `src/event.rs`
(the 9 design-doc desks as concrete event payloads).

**Biometric samples are structurally non-disclosable.** Per an explicit
decision, raw template bytes must never leave the device — `BiometricSample`
(`src/biometric.rs`) is deliberately not a `Claim`: it has no `claim_type`,
so `Grant`/`disclose_data` have no path to it at all. The only sanctioned
mesh operation on it is a match verdict against a live-captured probe, not
disclosure of the template — that verification flow (procedure shape, and
how to require genuine in-the-moment subject awareness rather than a
silent remote probe) isn't designed yet.

**Command/handler layer and SQLite persistence are built, 2026-08-28.**
`Store` (`src/store.rs`) is the append-only SQLite log; `Dossier`
(`src/dossier.rs`) replays a subject's stream into current state;
`handler::handle` (`src/handler.rs`) turns a `Command` into an event —
one `maybe_*` function per design-doc desk — or rejects it with a
`DomainError` before anything is appended. `disclose_data` is the one
exception: it never rejects, it decides `DataDisclosedV1` vs
`DataAccessDeniedV1` (with the specific reason: no matching grant,
expired, or revoked) against `Dossier::active_grants`, matching the
design doc's determinism rule — reads only the dossier's own replayed
state, nothing external. Covered by a full grant→use→deny→revoke
integration test (`tests/lifecycle.rs`) plus CBOR round-trip tests for
every event variant; `cargo test`, `cargo clippy --all-targets` both
clean.

Local commands (`initiate_passport` through `revoke_data_access`) check
state preconditions only, not caller authority — they only ever
originate from the app running locally on the subject's own device,
which is already the trust boundary. `disclose_data` alone crosses an
actual security boundary (a remote mesh party), which is why it alone is
decided against the dossier's grants rather than simply accepted.

**UniFFI bindings built, 2026-08-28** (`macula-passport-ffi/`, new
workspace member). `FfiPassport` is the one exported object: one method
per desk (`initiate`, `registerIdentityDocument`, `grantDataAccess`,
`discloseData`, ...), each doing load→replay→`handler::handle`→append as
a single call — the mobile side never manually replays. All decision
logic stays in the core crate exactly as planned; nothing here decides
anything, it only marshals `Command` in and `PassportEvent`/`DomainError`
out. Structure mirrors `macula-rust-sdk-ffi`'s relationship to
`macula-rust-sdk` (a separate crate, so the core stays UniFFI-free);
`FfiValue` mirrors that crate's own restricted CBOR-value mirror
(`Null`/`Int`/`Bytes`/`Text`/`Float` — no `List`/`Map` yet, same
limitation, not independently reinvented).

No Kotlin/Swift/mobile toolchain exists in this dev environment (same
constraint `macula-rust-sdk-ffi` has), so verification matches what that
repo's own CI settles for: real `uniffi-bindgen generate` runs for both
`--language kotlin` and `--language swift`, confirming the full API
surface (`discloseData`, `grantDataAccess`, `FfiDisclosureOutcome`, ...)
actually codegens with correct types and doc comments, not just that the
Rust side compiles. Plus a full grant→use→deny→revoke integration test
through the actual `FfiPassport` object (`macula-passport-ffi/tests/lifecycle.rs`),
separate from the core crate's own test of the same lifecycle, since the
FFI layer's own marshalling (byte-length validation, the post-disclosure
claim lookup) isn't exercised by the core tests at all. `cargo test
--workspace` and `cargo clippy --workspace --all-targets` both clean.

**Still open:** no CI workflow yet (the sibling SDK repos all run a
bindgen codegen smoke test in GitHub Actions; this repo doesn't have
Actions configured at all) — not built now, flagging as the natural next
piece rather than assuming it's wanted. No actual Kotlin/Swift consumer
app exists — the bindings are verified to generate correctly, not
run against a real mobile build (this dev box has no toolchain for that).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option, matching `macula-rust-sdk`'s convention (standard for the
Rust crate ecosystem this app's core is built in).
