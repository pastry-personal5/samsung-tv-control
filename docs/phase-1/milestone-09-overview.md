# P1-M9: Remote Command Admission Policy

Status: Draft

## Goal

Decide whether a typed remote-action request is eligible to proceed based on
the current selected device, Pairing, and Connection state.

## Sequence

Start after P1-M8. This milestone completes a pure policy boundary. It does
not send a command or report a successful TV action.

## Scope

In scope:

- Evaluate P1-M5 `SendRemoteAction` against the selected identity and
  generation-scoped lifecycle state.
- Return a distinct typed rejection for no selected TV, wrong target or stale
  selection generation, pairing required, and not connected.
- Return a policy-approved/admitted value only when the target is current,
  pairing is ready, and Connection is ready; the result means policy passed,
  not that transport accepted or sent anything.
- Keep remote-control UI disabled until the application projection says the
  current request is eligible; preserve accessible reasons for rejected
  actions.
- Add table-driven deterministic tests for each precondition and the passing
  policy case.

Out of scope:

- Bounded queues, request IDs, async execution, cancellation, I/O, transport
  results, capability probing, or claims about observed TV state.
- Enabling controls based solely on Pairing or on a selected identity.
- Sources, apps, wake, exact volume, or text commands.

## Completion checklist

- [ ] Every remote request is checked against the authoritative application
  snapshot before it can leave the policy boundary.
- [ ] Rejections are distinct and deterministic, with no accepted result for
  absent, stale, unpaired, or disconnected targets.
- [ ] The eligible result is clearly documented as a policy decision only;
  no network work or success message follows from it.
- [ ] Iced projects enabled state and accessible rejection reasons from the
  same application policy result.
- [ ] Tests cover each rejection, target/generation checks, and the eligible
  case without I/O.
- [ ] Run the required Cargo format, Clippy, and test gates from the
  [contribution guide](../contribution-guide.md); record evidence and mark the
  milestone Done.
