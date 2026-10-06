# P1-M8: Connection State Projection

Status: Draft

## Approach

Add typed lifecycle state to the application snapshot established in P1-M7.
Pairing and Connection are independent: pairing readiness does not imply a live
Connection, and a socket state does not prove pairing succeeded. In this
milestone the state can be advanced only through pure application methods used
by tests or later use cases; there is no I/O adapter to call them.

Every lifecycle update carries the selection generation it belongs to. Reject
updates for stale generations. When selected identity changes, reset its
Pairing and Connection projection to unknown/not connected so old status cannot
be presented for the new device.

## Module responsibilities

- `application::state`: own compact Pairing and Connection enums and bind them
  to the selected identity and generation.
- `application` transitions: validate generation and update lifecycle facts;
  return an explicit stale-generation result when an update does not apply.
- `presentation::iced::view_model`: map typed states to separate status labels
  and concise safe explanations; do not infer success from silence or from a
  selection change.
- `presentation::iced::view`: render both statuses in the existing status
  region without moving the controls.

Avoid timestamps, event streams, retry policy, and monitoring infrastructure
until a real adapter can supply those facts.

## Sequencing

1. Add lifecycle types and generation-scoped state transitions.
2. Add deterministic tests for all defined transitions, stale updates, and
   selection-change reset behavior.
3. Add the presentation projection and verify Pairing and Connection remain
   visually distinct.
4. Run the Cargo gates. No Iced subscription or socket operation is added.

P1-M9 uses these states as preconditions for deciding whether a typed remote
request can be admitted.
