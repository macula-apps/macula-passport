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

**Still open:** every field-level schema (identity document types per
jurisdiction, biometric template formats, health-observation vocabulary,
consent-grant taxonomy) and the UniFFI binding shape.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option, matching `macula-rust-sdk`'s convention (standard for the
Rust crate ecosystem this app's core is built in).
