# P1-M8: Connection State Projection

Status: Draft

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

- [ ] Pairing and Connection have separate typed states and separate UI labels.
- [ ] State updates tied to an outdated selection generation are ignored or
  explicitly rejected.
- [ ] Selection changes reset the prior device's transient lifecycle state.
- [ ] Failure states map to safe, actionable text without leaking local device
  data.
- [ ] Tests cover state transitions and projection without network or TV use.
- [ ] Run the required Cargo format, Clippy, and test gates from the
  [contribution guide](../contribution-guide.md); record evidence and mark the
  milestone Done.
