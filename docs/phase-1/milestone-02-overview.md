# P1-M2: Initial Architecture

Status: Planned

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

- [ ] Publish a module and responsibility map.
- [ ] Define the UI-to-device command flow and error flow.
- [ ] Record storage and redaction rules for pairing data.
- [ ] Identify interfaces that permit deterministic tests without a TV.
- [ ] Link resolved decisions and remaining risks from the phase changelog.
