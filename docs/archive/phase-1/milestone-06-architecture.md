# P1-M6: No-Selected-TV Control Gate

Status: Archived

## Approach

P1-M6 adds the smallest useful application control gate: requests cannot
proceed when no `DeviceId` is selected. The initial state represents only
`Selection::None`; it does not create a fake saved device, infer a connection,
or provide a route around future pairing and trust checks.

The application exposes an optional selected-device projection, a typed
control status, and a pure method that evaluates the P1-M5 typed remote-action
request. In the unselected state, the method returns a typed `NoSelectedTv`
rejection with the original request. It must not
return an accepted/pending outcome, allocate a request ID, enqueue work, or
invoke a port. Later milestones will extend this same boundary with selection,
connection, pairing, and capability preconditions before any dispatcher or
transport is introduced.

## Module responsibilities

- `application::state`: own selection state and its Iced-free optional-device
  and control-status projections.
- `application::command`: retain the P1-M5 request value and add only the
  result/rejection type required to express the no-selection decision.
- `application` service/reducer: evaluate a remote-action request against the
  snapshot. Keep it synchronous and pure.
- `presentation::iced::message`: carry the typed remote-action intent without
  exposing Iced to the application.
- `presentation::iced::view_model`: retain a projection of the application
  snapshot and turn `NoSelectedTv` into the visible explanation used by
  Remote View.
- `presentation::iced::app` and `view`: pass intent into the pure gate and
  render the returned projection/result. They may publish a safe presentation
  message, but must not create a `Task` that contacts a TV.

The `App` composition root may construct the pure application state directly
for now. Do not define an artificial port or an async trait just to preserve a
future seam; a real port belongs with the first behavior that needs it.

## Sequencing

1. Add the application state, rejection result, and pure no-selection gate
   with unit tests.
2. Add the Iced message and ViewModel projection, preserving existing route,
   settings-window, splitter, and feed behavior.
3. Replace hard-coded Remote View availability text with the projection and
   cover the presentation reducer with deterministic tests.
4. Run the Cargo gates and visually verify that the initial screen remains
   unchanged in meaning: controls are visible, disabled, and explain how to
   select a TV.

This is a policy seam, not a partial device implementation. A future milestone
must add real selection and connection facts before any action can become
enabled.
