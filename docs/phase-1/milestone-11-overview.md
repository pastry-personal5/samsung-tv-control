# P1-M11: Clean Architecture Refactoring

Status: Planned

## Goal

Improve the implemented P1-M10 codebase's names, module boundaries, and test
layout without changing Samsung-TV behavior, saved-data compatibility, or the
user journey.

## Sequence

Start only after P1-M10 is marked Done. This is a behavior-preserving
refactoring milestone: it must not be used to finish deferred live-hardware
acceptance or to add sources, apps, text input, wake, Power Toggle, or exact
volume control.

## Scope

In scope:

- Establish the current implementation as the baseline: run the full Cargo
  gates, preserve the P1-M10 deterministic scenarios, and add narrow
  characterization tests where a rename or move would otherwise leave a
  behavior boundary unprotected.
- Rename modules, public types, functions, parameters, local variables, test
  names, and UI-facing identifiers when a name hides ownership, uses a
  deprecated product term, or conflates a request with an observed result.
- Apply the canonical terminology in [UX terms](../ux-term.md): use **TV** in
  user-facing copy; retain `device` only for saved-record and internal identity
  concepts; use `Enter` rather than `Select` for the center Directional Pad
  action; and retain the distinct meanings of Pairing, Connection, request
  outcome, and observed TV state.
- Clarify layer ownership and dependency direction. The domain remains pure;
  the application layer owns use-case policy and port traits; infrastructure
  implements ports and owns wire/platform detail; Iced only maps user intent
  and projects application facts.
- Group files by cohesive responsibility, using small `mod.rs` re-exports only
  where they create a simpler boundary. Move tests to mirror their source
  boundary when doing so makes ownership clearer.
- Remove obsolete aliases and compatibility shims within the crate once every
  call site has moved. Update architectural and contributor documentation to
  describe the resulting tree and terminology.

Out of scope:

- New product behavior, protocol mappings, requests, async/concurrency
  semantics, retry policy, persistence formats, Keychain item identities,
  bundle identity, or network dependencies.
- Modifying saved-TV, certificate-trust, or Keychain-token data. Existing app
  data must remain readable and use the same storage keys.
- Changing visible labels or owner-approved keyboard shortcuts except to fix a
  glossary inconsistency without changing what the control does.
- Broad formatting-only rewrites, unrelated dependency upgrades, or exposing a
  new public library API.

## Completion checklist

- [ ] The pre-refactor baseline and final tree have passing `cargo fmt --all
  -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
- [ ] Module ownership is documented and enforces the dependency direction:
  domain has no application/infrastructure/presentation dependency;
  application has no Iced, Samsung-wire, or macOS dependency; presentation has
  no storage or socket dependency; infrastructure has no presentation
  dependency.
- [ ] The canonical glossary is applied to user-facing strings, presentation
  identifiers, and semantic action names. In particular, the action rendered
  as **Enter** is named `Enter` at the domain/application boundary; Samsung's
  `KEY_ENTER` mapping remains an infrastructure detail.
- [ ] A request, admission decision, local write result, and observed TV state
  remain separately typed and cannot be conflated by a renamed API.
- [ ] The Samsung adapter remains the sole owner of Samsung `KEY_*` strings,
  codec parsing/encoding, and WebSocket session mechanics. No Iced or
  application module imports those details.
- [ ] Device-scoped trust and credentials retain their existing persistence
  behavior: secrets stay in Keychain, non-secret preferences stay separate,
  and no token/address/identifier appears in fixtures or logs.
- [ ] Selection generation, request ordering, queue capacity, cancellation,
  uncertain-write handling, and stale-event protections retain their existing
  deterministic coverage.
- [ ] Integration and unit tests use behavior-oriented names and mirror the
  final source ownership where practical; no automated test needs a TV or LAN.
- [ ] `README.md`, [planned repository architecture](../planned-repository-architecture.md),
  and relevant milestone documentation match the final names and boundaries.
- [ ] The changelog records the completed rename map, compatibility guarantees,
  validation evidence, and any deliberately retained legacy name with its
  reason. Then mark P1-M11 Done.

## Acceptance evidence

Record the baseline and final test counts, the final module map, and any
observable copy correction. For storage-bearing names, record that the change
was source-only and did not alter paths, JSON keys, Keychain service/account
values, pairing tokens, certificate pins, bundle ID, or network behavior.
