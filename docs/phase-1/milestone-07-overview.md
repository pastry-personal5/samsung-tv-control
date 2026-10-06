# P1-M7: In-Memory TV Selection

Status: Draft

## Goal

Let the application select or clear a known local device during the current
session, and identify each selection with a generation so stale work cannot be
mistaken for current-device state.

## Sequence

Start after P1-M6. The application may select only a `DeviceId` and safe
display information supplied by its caller. This milestone does not create
devices from network candidates or persist them.

## Scope

In scope:

- Add the smallest application-owned selected-device state using the P1-M5
  `DeviceId` and safe device display value.
- Support selecting a device and clearing the selection; selecting the
  already-selected device is an idempotent no-op.
- Increment a monotonic selection generation whenever the selected identity
  changes, including when selection is cleared.
- Project the selected device and generation into the Iced ViewModel without
  making the presentation layer authoritative.
- Add deterministic tests for selection, clearing, idempotence, and generation
  changes.

Out of scope:

- Creating or validating device records, address entry, discovery, persistence,
  startup restoration, trust, or pairing.
- Connection attempts, command dispatch, cancellation of real work, sockets,
  or Samsung protocol handling.
- Changing the settings page into a functional device picker.

## Completion checklist

- [ ] Selection state is owned by the application and exposed through a safe
  snapshot/projection.
- [ ] Selection, clearing, repeated selection, and generation changes have
  deterministic tests.
- [ ] Device display projections and diagnostics contain no host, MAC address,
  token, certificate, or raw protocol data.
- [ ] Iced can display the selected-device projection while the application
  remains the source of truth.
- [ ] Run the required Cargo format, Clippy, and test gates from the
  [contribution guide](../contribution-guide.md); record evidence and mark the
  milestone Done.
