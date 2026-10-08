# P1-M11: Clean Architecture Refactoring

Status: Planned

## Architectural intent

P1-M10 established a working vertical slice. P1-M11 makes its structure state
the ownership already intended by the architecture, without changing runtime
behavior. Work in reviewable slices: characterize a boundary, rename or move
one cohesive unit, update all callers and tests, run the relevant tests, then
continue. Do not combine a refactor with a behavioral fix; record any discovered
bug separately for a later decision.

The dependency graph after the refactor remains:

```text
presentation::iced  -> application <- infrastructure
                         ^              |
                         |              v
                       domain <---------+

main.rs: composition root only
```

`application` owns use cases and declares the ports that it needs.
`infrastructure` implements those ports. `main.rs` is the only place that
chooses concrete adapters. The diagram does not allow infrastructure to call
Iced or Iced to access a socket, Keychain, or preferences directly.

## Target module map

Use this map as the intended destination. Create a directory only when it
groups real related behavior; a thin module hierarchy is not a goal itself.

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
| `infrastructure::ssdp` | `infrastructure::ssdp_discovery` | Retain SSDP as an adapter/protocol qualifier, not an application concept. |
| `infrastructure::storage` | `infrastructure::preferences` | Make clear this is non-secret local persistence. |
| `infrastructure::keychain` | `infrastructure::macos::keychain` | Put the macOS-specific credential adapter behind a platform boundary. |
| `infrastructure::samsung::{codec,session}` | unchanged | These names accurately separate bounded wire codec and live WebSocket session. Keep `KEY_*` mapping internal to this subtree. |
| `presentation::iced::{app,message,view,view_model}` | `presentation::iced::{app,ui_message,view,view_model}` | Rename only the ambiguous `message` module; retain the compact, established Iced boundary. |

Module moves use `git mv`, followed by compiler-led import updates. Keep
crate-root re-exports intentionally small. Since this is an application rather
than a stable library, remove old in-crate aliases rather than maintaining two
vocabularies. If a public re-export is needed by an integration test, re-export
the final canonical type only.

## Name and type rules

- Use `tv` for a product-facing TV or UI value: `selected_tv_label`,
  `discovered_tvs`, and `tv_address`. Use `device` for a persisted record,
  opaque identity, or repository operation: `DeviceId`, `SavedDevice`, and
  `DeviceRepository`.
- Rename `Select` to `Enter` at every semantic-action call site, test, shortcut
  mapper, label helper, capability set, and application result. The only
  remaining word `select` should mean choosing a Saved TV/candidate/row, not
  pressing the Directional Pad center.
- Name values by role rather than representation: `remote_request`,
  `selected_device_id`, `selection_generation`, `certificate_pin`,
  `pairing_token`, `terminal_result`, and `connection_attempt`. Avoid generic
  names such as `data`, `item`, `result`, `state`, and `value` when a more
  specific name is locally available.
- Preserve the outcome vocabulary: a request is admitted or rejected; an
  admitted request is queued; its terminal transport result is not sent,
  written, failed, or uncertain; only a TV report becomes observed TV state.
  Do not rename a local write to `sent`, `successful`, or `confirmed`.
- Keep user-visible labels exactly aligned to `docs/ux-term.md`: **Remote
  View**, **Settings Window**, **TV settings**, **Global Messages Pane**,
  **Activity View**, **Pairing**, **Connection**, and **Enter**.

## Safe execution order

1. Capture the baseline Cargo gates and identify imports, public re-exports,
   persistence constants, and integration-test dependencies. Add
   characterization coverage only for untested behavior being moved.
2. Rename the domain action (`Select` → `Enter`) and its semantic callers. Run
   action, codec, dispatcher, and shortcut tests. The Samsung adapter still
   maps `RemoteAction::Enter` to `KEY_ENTER`.
3. Rename application ports and use-case modules in cohesive batches. Keep
   their error semantics and method contracts unchanged. Refactor imports with
   compiler assistance; do not alter storage keys or flow ordering.
4. Move infrastructure adapters behind descriptive/proper platform boundaries.
   Test codec/session, preference, certificate, and Keychain fakes after each
   move. Do not move protocol constants outside `infrastructure::samsung`.
5. Clarify Iced message and projection names; ensure the UI continues to issue
   only typed application requests and receives only typed facts/outcomes.
6. Reorganize tests by final owner where that improves clarity, then update the
   repository architecture, README, phase notes, and changelog. Run the full
   gates and inspect `git diff --check` before completing.

## Invariants and compatibility

The refactor must preserve `SavedDevice` serialization and preferences path,
certificate-trust records, Keychain service/account naming, selection and
request-generation semantics, queue capacity/order, TLS pin checks, 8002-only
transport, retry limits, and bundle identity. Add a regression test around any
of these invariants if existing coverage does not make preservation evident.

No live TV exercise is required merely for a source-only refactor, but no
hardware compatibility claim may be broadened. The P1-M10 sanitized hardware
matrix remains the source of live-support evidence.
