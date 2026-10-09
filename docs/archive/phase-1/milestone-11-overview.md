# P1-M11: Clean Architecture Refactoring

Status: Archived

## Goal

Move live-control orchestration out of Iced, clarify names and module
boundaries, and align the stored remote-action name with **Enter**. Preserve
Samsung-TV control behavior. The owner does not require compatibility with
existing saved records or older app builds.

## Sequence

P1-M10 is Done by the owner's closure decision. Its deferred hardware checks
remain unverified and do not become P1-M11 acceptance evidence. P1-M11 adds no
sources, apps, text input, wake, Power Toggle, or exact volume control.

## Scope

In scope:

- Establish the current implementation as the baseline: run the full Cargo
  gates, preserve the P1-M10 deterministic scenarios, and add narrow
  characterization tests where a rename or move would otherwise leave a
  behavior boundary unprotected.
- Rename modules, public types, functions, parameters, local variables, test
  names, and UI-facing identifiers when a name hides ownership, uses a
  deprecated product term, or conflates a request with an observed result.
- Apply the canonical terminology in [UX terms](../initial-ux-term.md): use **TV** in
  user-facing copy; retain `device` only for saved-record and internal identity
  concepts; use `Enter` rather than `Select` for the center Directional Pad
  action; and retain the distinct meanings of Pairing, Connection, request
  outcome, and observed TV state.
- Extract startup, discovery/probe, Pairing/re-pair, reconnect, session
  lifecycle, remote dispatch, cancellation, and stale-result coordination from
  `presentation::iced::app` into an application coordinator. The application
  owns use-case policy and narrow ports; infrastructure implements the ports
  and owns wire/platform detail. `main.rs` constructs the adapters. Iced maps
  user intent, schedules effects, and projects typed application results.
- Rename the serialized `verified_actions` value from `Select` to `Enter` for
  newly written `devices.json` records. Do not add a legacy reader or a data
  migration. A pre-M11 record containing `Select` may be unreadable; document
  how the owner can move old preferences aside and set up the TV again. Do not
  silently overwrite an unreadable file or delete existing Keychain items.
- Group files by cohesive responsibility, using small `mod.rs` re-exports only
  where they create a simpler boundary. Move tests to mirror their source
  boundary when doing so makes ownership clearer.
- Remove obsolete Rust aliases once every call site has moved. Update
  architectural and contributor documentation to describe the resulting tree,
  terminology, and saved-data break.

Out of scope:

- New product behavior, protocol mappings, request or retry policy, ordering,
  queue limits, Keychain item identities, bundle identity, or network
  dependencies. The planned persisted-data break is the action value
  `Select` → `Enter` in saved `verified_actions`.
- Modifying certificate-trust or Keychain-token data, or other saved-TV fields.
- Backward reading, migration, or automatic deletion of old saved records.
- Changing visible labels or owner-approved keyboard shortcuts except to fix a
  glossary inconsistency without changing what the control does.
- Broad formatting-only rewrites, unrelated dependency upgrades, or exposing a
  new public library API.

## Completion checklist

- [x] The pre-refactor baseline and final tree have passing `cargo fmt --all
  -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
- [x] Module ownership is documented and enforces the dependency direction:
  domain has no application/infrastructure/presentation dependency;
  application has no Iced, Samsung-wire, or macOS dependency; presentation has
  no storage or socket dependency; infrastructure has no presentation
  dependency.
- [x] `main.rs` composes concrete adapters; Iced has no direct imports of
  `infrastructure`, Samsung session types, Keychain, preferences, or socket
  channels. Application-owned tests exercise the extracted workflow through
  fake ports.
- [x] The canonical glossary is applied to user-facing strings, presentation
  identifiers, and semantic action names. In particular, the action rendered
  as **Enter** is named `Enter` at the domain/application boundary; Samsung's
  `KEY_ENTER` mapping remains an infrastructure detail.
- [x] A fresh saved TV serializes `RemoteAction::Enter` as `"Enter"` and loads
  correctly. An old file containing `"Select"` returns a clear storage error
  without deletion or overwrite. The reset and re-pair procedure is documented
  before P1-M11 is marked Done.
- [x] Requests, admission decisions, and local write results remain distinct
  types. No observed TV state is inferred from a successful socket write.
- [x] The Samsung adapter remains the sole owner of Samsung `KEY_*` strings,
  codec parsing/encoding, and WebSocket session mechanics. No Iced or
  application module imports those details.
- [x] Device-scoped trust and credentials retain their existing persistence
  behavior: secrets stay in Keychain, non-secret preferences stay separate,
  and no real token, LAN address, or device identifier appears in fixtures or
  logs.
- [x] Selection generation, request ordering, queue capacity, cancellation,
  uncertain-write handling, stale-event protections, observer reattachment,
  and one terminal result per admitted request retain deterministic coverage.
- [x] Integration and unit tests use behavior-oriented names and mirror the
  final source ownership where practical; no automated test needs a TV or LAN.
- [x] `README.md`, [repository architecture](../planned-repository-architecture.md),
  and relevant milestone documentation match the final names and boundaries.
- [x] The changelog records the completed rename map, saved-data break,
  validation evidence, and any deliberately retained legacy name with its
  reason. Then mark P1-M11 Done.

## Acceptance evidence

The baseline passed all three Cargo gates with 92 tests (75 unit, 17
integration). The final tree passes those same gates with 97 tests (80 unit,
17 integration). The [repository architecture](../planned-repository-architecture.md)
records the final module map. Focused tests cover fresh `"Enter"` storage
round-trip, safe rejection of an old `"Select"` file without modification,
and fake-port coordinator handling of stale completions, token rejection, and
re-pair generations. The user-facing startup message explains that unreadable
Saved TV settings must be backed up before pairing again.

The only on-disk change is the serialized `verified_actions` value. Pre-M11
`devices.json` files containing `Select` are unsupported and may require the
[manual backup and re-pair procedure](milestone-11-architecture.md#breaking-saved-action-spelling).
Keychain service/account values, pairing tokens, certificate pins, bundle ID,
and network behavior remain outside this refactor. No live-TV validation was
performed for M11; the [P1-M10 hardware matrix](milestone-10-hardware-matrix.md)
continues to limit hardware support claims.
