# P1-M2: Initial Architecture

Status: Archived

## Proposed boundaries

The initial architecture is recorded in the
[software architecture](../initial-software-architecture.md) and
[planned repository architecture](../planned-repository-architecture.md). It
separates:

- macOS presentation and input handling;
- application state and command orchestration;
- Samsung TV discovery, connection, pairing, and remote-command transport;
- persistent pairing-token storage; and
- transport fakes used by unit and integration tests.

## Decisions recorded

- Iced is the selected macOS window framework, with a Rust-owned application
  and protocol core.
- The first awake-TV adapter will use the community-observed local Tizen
  WebSocket protocol; exact endpoint and supported models require hardware
  validation.
- Pairing tokens belong in macOS Keychain; logs and diagnostics are sanitized.
- The application dispatcher owns command ordering and cancellation policy;
  one session owner serializes socket I/O.
- Discovery includes local-network discovery and manual host entry; the
  discovery mechanism remains subject to target-TV validation.
- First usable app scope includes core controls, wake, source selection,
  installed-app launch, and text input where supported.
- Initial installation targets the owner's Mac. Port 8001 remains an open
  compatibility decision pending hardware tests.

## Sequencing

The architecture milestone is complete. Hardware compatibility questions
remain tracked in the software architecture and research notes for resolution
during implementation and device testing.
