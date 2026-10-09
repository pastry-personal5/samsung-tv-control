# P1-M6: No-Selected-TV Control Gate

Status: Archived

## Goal

Replace the shell's presentation-owned no-TV placeholder with a minimal,
application-owned control gate and project its explicit disabled reason into
Remote View.

## Sequence

Start after P1-M5. This milestone intentionally models only the initial state:
there is no selected TV. It establishes the seam for future selection and
connection work without pretending that a device can be used.

## Scope

In scope:

- Add a small application control-state projection that exposes an optional
  selected `DeviceId`, plus a typed result for an attempted remote-action
  request.
- Define the no-selection policy: every remote action is rejected before
  admission with a specific, user-presentable `NoSelectedTv` reason; no queue,
  task, subscription, or I/O starts.
- Give the Iced `ViewModel` a projection of that application state and derive
  Remote View's controls and disabled explanation from it.
- Route a Remote View button message through the presentation reducer to the
  application gate. The current disabled buttons must remain non-dispatching;
  the reducer path exists for deterministic tests and future enabled controls.
- Map the typed no-selection rejection to a safe local status/message without
  inventing a TV result or exposing device/network data.
- Add deterministic tests for the gate, its target-preserving rejection, and
  the view-model's no-selected-TV projection.

Out of scope:

- Selecting, saving, restoring, or forgetting a TV.
- Pairing, trust, connection, reconnecting, capabilities, activity outcomes,
  a command queue, cancellation, or dispatch to a transport.
- Discovery/manual address entry, Keychain/preferences, local-network access,
  Samsung protocol frames, and any successful remote action.
- Changing P1-M4 navigation, multi-window ownership, message-feed behavior,
  or visual styling beyond replacing its hard-coded no-TV wording.

## Completion checklist

- [x] The application owns a small state/snapshot whose initial value has no
  selected TV and exposes that fact without an Iced dependency.
- [x] An attempted typed remote-action request with no selected TV deterministically
  returns the distinct `NoSelectedTv` rejection and cannot start I/O.
- [x] Remote View derives its disabled control state and visible reason from
  the application projection rather than a locally hard-coded assumption.
- [x] The presentation reducer can receive a typed remote-action intent and
  safely renders the no-selection result; disabled buttons do not emit it.
- [x] Unit tests cover the gate and presentation projection without a TV,
  network, Iced runtime, or macOS service.
- [x] Run the required Cargo format, Clippy, and test gates from the
  [contribution guide](../../contribution-guide.md), then record the evidence in
  the phase changelog and mark this milestone Done.

## Acceptance evidence (2026-10-08)

`State::none()` rejects every typed remote action before dispatch and returns
the original request with `NoSelectedTv`. The Iced reducer accepts the request
value without constructing a synthetic target, and publishes a safe “not sent”
message. Native macOS review confirmed the no-TV status and disabled controls
remain visible. The Cargo format, Clippy, and test gates passed (33 tests).
Native screen-reader control names remain the product follow-up recorded in
the [roadmap](../../roadmap.md).
