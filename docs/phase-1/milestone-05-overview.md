# P1-M5: Typed Control Contracts

Status: Done

## Goal

Establish a compact, pure Rust vocabulary for the device identity and semantic
remote-control intent that the Iced shell can eventually submit to the
application layer.

## Sequence

Start after P1-M4. P1-M5 is a contract-only milestone: its types must compile
and be deterministically testable without a TV, local network, macOS service,
or running Iced window.

## Scope

In scope:

- Add the minimal `domain` and `application` module exports needed by the
  contracts, including a library target for integration tests.
- Define an opaque local `DeviceId` and a small device display projection that
  contains no address, pairing token, certificate, or Samsung-protocol data.
- Define a finite semantic `RemoteAction` set for the controls already visible
  in Remote View: power toggle, directional navigation, select, back, home,
  mute, and volume up/down.
- Define one typed application request for dispatching a semantic remote action
  to a specific `DeviceId`, plus explicit construction/accessor APIs required
  by the next milestone.
- Unit-test identity equality, action coverage, and the command's preservation
  of its typed target and action.

Out of scope:

- Device selection, connection, pairing, capability checks, command admission,
  queueing, transport I/O, protocol keys, or any Samsung frame.
- Sources, installed apps, exact volume values, text entry, discovery, wake,
  storage, Keychain, or macOS APIs.
- Wiring a control button to the application contract; that belongs to P1-M6.

## Completion checklist

- [x] `src/lib.rs` exposes only the necessary domain and application modules
  for deterministic tests; the executable composition root remains separate.
- [x] A `DeviceId` represents a local saved-record identity and cannot be
  confused with a host, MAC address, advertised TV name, token, or certificate.
- [x] `RemoteAction` is finite and covers every currently rendered enabled-or-
  disabled button action without accepting arbitrary Samsung key strings.
- [x] The typed remote-action request always carries both a `DeviceId` and a
  `RemoteAction`; its public API does not expose wire-frame construction.
- [x] Focused unit or integration tests verify the public contract without a
  TV, network, or macOS service.
- [x] Run the required Cargo format, Clippy, and test gates from the
  [contribution guide](../contribution-guide.md), then record the evidence in
  the phase changelog and mark this milestone Done.

## Validation evidence

```
$ cargo fmt --all -- --check
# Passed (no output)

$ cargo clippy --all-targets -- -D warnings
# Passed (no warnings)

$ cargo test
# 22 passed; 0 failed
```
