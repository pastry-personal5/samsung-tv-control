# P1-M2: Initial Architecture

Status: Planned

## Proposed boundaries

The architecture will be specified after P1-M1 completes. At minimum, it must
separate:

- macOS presentation and input handling;
- application state and command orchestration;
- Samsung TV discovery, connection, pairing, and remote-command transport;
- persistent pairing-token storage; and
- transport fakes used by unit and integration tests.

## Decisions to make

- The macOS UI framework and its Rust integration boundary.
- The Samsung protocol client strategy and supported device versions.
- Token storage location, encryption, and redaction policy.
- Async/concurrency ownership and cancellation behavior.

## Sequencing

Use the P1-M1 findings to choose these boundaries. Document the selected
approach, affected modules, public interfaces, and implementation order before
starting the next phase.
