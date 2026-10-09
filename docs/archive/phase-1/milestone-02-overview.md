# P1-M2: Initial Architecture

Status: Archived

## Goal

Turn P1-M1 research into an implementable initial architecture for the native
macOS Rust application.

## Scope

In scope:

- Rust crate and module boundaries.
- The boundary between macOS UI, application state, and Samsung TV transport.
- Error handling, pairing-token storage, and test seams.
- A sequence for the first implementation milestone.

Out of scope:

- Full product implementation or UI polish.
- Premature optimization unsupported by the P1-M1 findings.

## Completion checklist

- [x] Publish a module and responsibility map in the
  [planned repository architecture](../planned-repository-architecture.md).
- [x] Define the UI-to-device command flow and error flow in the
  [software architecture](../initial-software-architecture.md).
- [x] Record storage and redaction rules for pairing data in the software
  architecture and [security research](../research/samsung-tv-protocol-security.md).
- [x] Identify interfaces that permit deterministic tests without a TV in
  the software and repository architectures.
- [x] Link resolved owner decisions and remaining hardware-validation risks
  from the [phase changelog](../../phase-1/changelog.md).

The implementation boundary and initial product scope are defined. Exact TV
endpoint, certificate, key, discovery, and wake behavior remain hardware
validation items and do not block this architecture milestone.
