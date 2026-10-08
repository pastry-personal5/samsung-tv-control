# P1-M11: Clean Architecture Refactoring

Status: Done

## Architectural intent

P1-M10 established a working vertical slice. P1-M11 gives application policy
ownership of the workflow currently concentrated in
`presentation::iced::app`. Preserve runtime control behavior while changing
the stored action value from `Select` to `Enter`. Work in reviewable slices:
characterize a boundary, move one cohesive responsibility, update callers
and tests, and run relevant checks. Record any unrelated bug separately.

The dependency graph after the refactor remains:

```text
presentation::iced  -> application <- infrastructure
                         ^              |
                         |              v
                       domain <---------+

main.rs: composition root only
```

`application` owns use cases, session/selection generations, cancellation,
dispatch, and typed facts/outcomes; it declares the ports it needs.
`infrastructure` implements those ports and owns sockets, Samsung frames,
SSDP, files, and Keychain. `main.rs` chooses concrete adapters and passes an
application coordinator to Iced. Iced owns window state, keyboard/focus
handling, Iced tasks/subscriptions, and display projections; it receives
application effect completions and session events as application types.

Before P1-M11, `presentation::iced::app` constructed the repository, Keychain,
and trust adapters; owned the dispatcher and active runtime; and coordinated
Pairing, reconnect, cancellation, and Samsung session events. Its UI message
also carried a concrete Samsung session. P1-M11 moved those responsibilities
behind application ports and the coordinator.

## Target module map

This map records the completed moves. Directories group real related behavior;
a thin module hierarchy is not a goal itself.

| Current responsibility | Target boundary | Notes |
| --- | --- | --- |
| `domain::device` | `domain::device` | Keep `DeviceId` and saved-record display values. `device` is correct internal vocabulary. |
| `domain::remote_action` | `domain::remote_action` | Rename `RemoteAction::Select` to `RemoteAction::Enter`; it is the semantic action, not the Samsung key. |
| `application::command` | `application::remote_request` | Name the typed remote request, its policy decision, and rejection around their actual responsibility. |
| `application::state` | `application::control_state` | Distinguish authoritative application state from Iced's view model/control projection. |
| `application::dispatcher` | `application::remote_dispatcher` | Keep the sole bounded FIFO and terminal outcomes here. |
| `application::device` | `application::device_repository` | Keep saved-device persistence port and errors together. |
| `application::discovery` | `application::tv_discovery` | Make the port's TV-candidate role explicit. |
| `application::secret` | `application::credential_store` | It stores pairing credentials, not arbitrary secrets. |
| `application::trust` | `application::certificate_trust` | Make the certificate-specific boundary explicit. |
| `application::target` | `application::tv_address` | Keep validation for a host/address target; do not imply URL or port selection. |
| `application::device_service` | `application::tv_setup_service` | Own save/select, Pairing commit, re-pair, reconnect material, and forget orchestration. |
| Iced-owned live workflow | `application::tv_control_coordinator` | Own startup selection, attempts/generations, discovery/probe/Pairing/reconnect orchestration, active-session policy, dispatch, and cancellation. Keep pure decisions testable through fake ports. |
| Direct Samsung session calls from Iced | `application::tv_session` port and `infrastructure::samsung::session` adapter | Expose typed probe, Pairing/connect, write, close, and session events; keep WebSocket/socket/channel types inside the adapter. |
| `infrastructure::ssdp` | `infrastructure::ssdp_discovery` | Retain SSDP as an adapter/protocol qualifier, not an application concept. |
| `infrastructure::storage` | `infrastructure::preferences` | Make clear this is non-secret local persistence. |
| `infrastructure::keychain` | `infrastructure::macos::keychain` | Put the macOS-specific credential adapter behind a platform boundary. |
| `infrastructure::samsung::{codec,session}` | unchanged | These names accurately separate bounded wire codec and live WebSocket session. Keep `KEY_*` mapping internal to this subtree. |
| `presentation::iced::{app,message,view,view_model}` | `presentation::iced::{app,ui_message,view,view_model}` | `ui_message` carries UI intent and application effect results, never a concrete Samsung session. `app` retains window and Iced task plumbing; `view_model` projects coordinator snapshots. |

Module moves kept implementations and tests together, followed by
compiler-led import updates. Crate-root re-exports expose the final canonical
types only. No legacy `Select` storage decoder was added.

## Coordinator contract

- `main.rs` constructs the concrete discovery, session, repository, trust,
  and credential adapters and injects them into the coordinator. No adapter
  is constructed or imported by `presentation::iced`.
- The coordinator accepts typed intentions such as discover, probe, confirm
  and pair, connect, select, forget, and send remote action. It returns typed
  snapshots, effects to run, and outcomes/events. Iced may schedule those
  effects, but cannot choose a TV host for a token, start a socket write,
  advance a selection generation, or interpret a Samsung session event.
- The session port carries typed application actions and results. Its adapter
  owns socket channels, task handles, TLS pin enforcement, port 8002, and
  Samsung error translation. Do not add a second queue: the application
  dispatcher remains the sole bounded remote-action FIFO.
- Preserve the current ordering of admission, dequeue recheck, write,
  terminal recording, and acknowledgment. Preserve attempt IDs and session
  IDs so late completions cannot revive a forgotten or newly selected TV.
  Closing an in-flight write remains uncertain; queued work is not sent.
- Keep the existing session-only message feed and window focus behavior in
  presentation. Application outcomes can supply safe facts; Iced chooses the
  display text. A candidate host and observed fingerprint may cross as typed
  display values for identity/trust confirmation. Pairing tokens, raw
  certificates, and wire payloads must not enter UI messages or logs; do not
  log real addresses or device identifiers.

## Breaking saved-action spelling

`devices.json` currently serializes `SavedDevice.verified_actions` through
the domain `RemoteAction` enum. Renaming the Rust variant to `Enter` makes new
records write `"Enter"`; old records containing `"Select"` will fail to load.
The owner accepts this break and requires neither a decoder nor migration.
Keep the storage error explicit and leave an unreadable `devices.json`
untouched. Do not automatically reset the file or remove old trust/Keychain
records. Before completion, document a recoverable manual procedure for
moving old preferences aside and pairing again. Note that old device-scoped
trust and Keychain items may then remain orphaned; cleanup requires a separate
owner decision.

For the current test installation, the manual recovery is: quit the app,
move `~/Library/Application Support/Samsung TV Remote/devices.json` to a
backup location outside that directory, relaunch, and pair the TV as a new
Saved TV. Keep the moved file recoverable. Do not remove `trust.json` or
Keychain items as part of this procedure.

Test a fresh record with `"Enter"` and a synthetic old record with `"Select"`.
The former must round-trip; the latter must fail without overwriting the
original file. The serialization change is the planned on-disk break.

## Name and type rules

- Use `tv` for a product-facing TV or UI value: `selected_tv_label`,
  `discovered_tvs`, and `tv_address`. Use `device` for a persisted record,
  opaque identity, or repository operation: `DeviceId`, `SavedDevice`, and
  `DeviceRepository`.
- Rename `Select` to `Enter` at every semantic-action call site, test, shortcut
  mapper, label helper, capability set, and application result. `Select`
  remains only in historical documentation and the test fixture proving the
  old file fails safely; lower-case `select` means choosing a Saved
  TV/candidate/row.
- Name values by role rather than representation: `remote_request`,
  `selected_device_id`, `selection_generation`, `certificate_pin`,
  `pairing_token`, `terminal_result`, and `connection_attempt`. Avoid generic
  names such as `data`, `item`, `result`, `state`, and `value` when a more
  specific name is locally available.
- Preserve the outcome vocabulary: a request is admitted or rejected; an
  admitted request is queued; its terminal transport result is not sent,
  written, or uncertain; only a TV report becomes observed TV state.
  Do not rename a local write to `sent`, `successful`, or `confirmed`.
- Keep user-visible labels exactly aligned to `docs/ux-term.md`: **Remote
  View**, **Settings Window**, **TV settings**, **Global Messages Pane**,
  **Activity View**, **Pairing**, **Connection**, and **Enter**.

## Safe execution order

1. Capture baseline Cargo gates, test counts, current imports, re-exports,
   storage constants, and deterministic scenarios. Record the current
   `Select` spelling and the accepted saved-data break.
2. Rename the domain action and semantic callers directly. Verify fresh
   `"Enter"` persistence and safe failure for an old `"Select"` file, plus
   Samsung `KEY_ENTER`, shortcuts, admission, and dispatch before proceeding.
3. Define typed application ports and coordinator outcomes. Extract startup,
   saved-TV selection, discovery/probe, and Pairing/re-pair/reconnect/forget
   flow in small slices. Move each associated stale-attempt and cancellation
   test from Iced to fake-port application tests as ownership changes.
4. Move active-session and bounded dispatch orchestration into the
   coordinator. Adapt the Samsung session to the port; remove concrete Samsung
   session packages, commands, events, and socket task handles from Iced types.
   Iced may carry application-owned opaque session and event types as effect
   results.
   Test queue capacity/order, one terminal result, uncertain writes, session
   replacement, and observer reattachment after each slice.
5. Wire the coordinator in `main.rs`, leaving Iced to map user input and
   render snapshots/outcomes. Rename cohesive modules and move macOS Keychain
   under `infrastructure::macos`. Confirm that no presentation source imports
   `infrastructure` or transport channel types.
6. Reorganize remaining tests by owner; update the repository architecture,
   README, phase notes, and changelog to match the implemented tree. Run the
   full gates and `git diff --check` before completing.

## Invariants and data break

The `verified_actions` spelling breaks existing saved records that contain
`Select`. Preserve runtime selection and request-generation semantics, queue
capacity/order, TLS pin checks, 8002-only transport, retry limits, and bundle
identity. Leave certificate-trust records and Keychain service/account values
untouched. Preserve behavior under cancellation and delayed completion when
moving the workflow. Add focused regression coverage where the baseline does
not prove an invariant.

No live TV exercise is required to complete this refactor, but no hardware
compatibility claim may be broadened. The P1-M10 sanitized hardware matrix
remains the source of live-support evidence and its deferred checks stay
deferred.
