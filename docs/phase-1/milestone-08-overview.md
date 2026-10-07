# P1-M8: Connection State Projection

Status: Complete

## Goal

Represent Pairing and Connection as distinct application-owned states and
project safe status text into the existing shell.

## Sequence

Start after P1-M7. This milestone defines the lifecycle vocabulary and
presentation mapping only. It does not attempt a Connection or assert that any
TV is online.

## Scope

In scope:

- Define small typed Pairing and Connection state enums, including initial,
  in-progress, ready, and recoverable failure states needed by the existing
  UX plan.
- Associate the status with the selected-device generation so a state update
  from an old selection cannot overwrite the current projection.
- Define pure state transitions for starting and completing an attempt,
  failure, and selection change invalidation.
- Project distinct Pairing and Connection labels and safe recovery guidance
  into the Remote View status region.
- Add deterministic tests for transitions, selection-generation mismatch, and
  presentation mapping.

Out of scope:

- Pairing or connection network operations, tokens, trust confirmation,
  reconnect timers, monitoring subscriptions, and protocol events.
- Claiming `Connected` from a caller-provided UI event; only a typed
  application transition may update the projection.
- Persistent state or background retries.

## Completion checklist

- [x] Pairing and Connection have separate typed states and separate UI labels.
- [x] State updates tied to an outdated selection generation are ignored or
  explicitly rejected.
- [x] Selection changes reset the prior device's transient lifecycle state.
- [x] Failure states map to safe, actionable text without leaking local device
  data.
- [x] Tests cover state transitions and projection without network or TV use.
- [x] Run the required Cargo format, Clippy, and test gates from the
  [contribution guide](../contribution-guide.md); record evidence and mark the
  milestone Done.

## Acceptance evidence (2026-10-08)

The application now owns independent `PairingState` and `ConnectionState`
facts for the current selection generation. Updates for prior generations and
updates with no current selection are explicitly ignored. Selecting another TV
or clearing the selection resets both facts. The Iced projection provides
separate labels and local recovery guidance, without any connection attempt,
token, or device data. Cargo format, Clippy, and test gates passed (38 tests).
