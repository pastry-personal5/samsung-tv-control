# P1-M5: Typed Control Contracts

Status: Archived

## Approach

Introduce only value types and their namespace boundaries. The contract is
semantic: presentation can express an intent such as `RemoteAction::Select`,
but no type in this milestone names a Samsung `KEY_*` value, endpoint, or wire
frame. That mapping remains the future infrastructure adapter's responsibility.

`src/lib.rs` exposes `domain` and `application` for testable library use while
`main.rs` remains the executable composition root. The modules must preserve
the dependency direction already recorded in the
[software architecture](../initial-software-architecture.md): `application` may depend on
`domain`; neither may import Iced or infrastructure.

## Module responsibilities

- `domain::device`: define the opaque `DeviceId` and a deliberately safe device
  display value. The identifier names the application's local saved record; it
  is not a network identity.
- `domain::remote_action`: define the closed set of semantic button intentions
  needed by the current Remote View. Group directional actions in a small
  nested enum only if it improves exhaustive matching; do not introduce generic
  string-based actions.
- `application::command`: define the typed request carrying a target
  `DeviceId` and `RemoteAction`. It is a request value, not a queue, future,
  result, or transport handle.
- `domain::mod`, `application::mod`, and `lib.rs`: publish the smallest stable
  surface needed by P1-M6 and its tests.

Use constructors and accessors that make invalid or misleading identity use
harder. The exact backing representation of `DeviceId` is private; no random
ID generation is needed yet because device creation and persistence are out of
scope. Keep display labels bounded or validation-ready, but do not add a
dependency merely to validate a label that no adapter accepts yet.

## Sequencing

1. Add the library/module skeleton and the domain value types.
2. Add the semantic action enum and application request type.
3. Add contract tests that exercise only public behavior.
4. Run the Cargo gates. Do not modify Iced views or add an infrastructure
   placeholder in this milestone.

P1-M6 will consume these contracts to make the shell's current no-selected-TV
state policy-driven. A later milestone may add selection, connection state,
capability checks, and a bounded dispatcher; none should be anticipated with
unimplemented traits or empty adapter modules here.

## Files created

- `src/domain/device.rs` - Opaque `DeviceId` and `DeviceDisplay` types
- `src/domain/remote_action.rs` - Finite `RemoteAction` enum
- `src/application/command.rs` - `SendRemoteAction` typed command
- `src/domain/mod.rs` - Domain module exports
- `src/application/mod.rs` - Application module exports
- `src/lib.rs` - Library root exposing domain and application
- `tests/integration.rs` - Integration test harness
- `tests/integration/device_id.rs` - DeviceId tests
- `tests/integration/remote_action.rs` - RemoteAction tests
- `tests/integration/command.rs` - Typed remote-action request tests

## Changes from architecture doc

The original P1-M5 architecture doc stated "Status: Draft". This milestone is
now `Done` with all contracts implemented and validated.
