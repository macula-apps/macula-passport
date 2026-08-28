# macula-passport

A sovereign identity, biometric, and health data vault — for a human or an
animal in someone's care — event-sourced **on the subject's own device**,
disclosed to other parties only through an explicit, per-request,
consent-gated exchange. Never broadcast, never stored anywhere but here.

## Status: design, not code

This repo is a container, not a build yet. No Cargo workspace, no UniFFI
bindings, no domain code — those need real decisions first (storage engine,
field-level schemas for identity/biometric/health data, disclosure
protocol shape), not placeholders standing in for them.

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

**Still open:** the embedded storage/event-log substrate (leaning SQLite
via `rusqlite`, already present on both target platforms, over a bundled
Rust KV engine or a full ES framework crate — not yet decided), and every
field-level schema (identity document types per jurisdiction, biometric
template formats, health-observation vocabulary, consent-grant taxonomy).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option, matching `macula-rust-sdk`'s convention (standard for the
Rust crate ecosystem this app's core is built in).
