# Repository architecture

This is the implemented P1-M11 layout for the Rust macOS application. It maps
the [software architecture](architecture.md) to current directories and files.
Later source, app, text, wake, and exact-volume features will add modules when
their behavior is implemented.

```text
.
├── Cargo.toml                    # package and dependency configuration
├── README.md
├── AGENTS.md
├── src/
│   ├── main.rs                   # executable entry and composition root
│   ├── lib.rs                    # module exports used by integration tests
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── device.rs             # stable local identity and display value
│   │   └── remote_action.rs      # finite semantic action set
│   ├── application/
│   │   ├── mod.rs
│   │   ├── remote_request.rs     # typed remote requests and admission policy
│   │   ├── control_state.rs      # selection, Pairing, Connection, availability
│   │   ├── remote_dispatcher.rs  # bounded FIFO and terminal-result journal
│   │   ├── tv_control_coordinator.rs # live use-case workflow and session policy
│   │   ├── tv_session.rs         # transport port and typed facts/events
│   │   ├── tv_discovery.rs       # candidate-discovery port
│   │   ├── tv_address.rs         # local host validation
│   │   ├── tv_setup_service.rs   # trusted setup, reconnect and forget
│   │   ├── device_repository.rs  # saved-record port and type
│   │   ├── credential_store.rs   # pairing credential port
│   │   └── certificate_trust.rs  # device-scoped trust port
│   ├── presentation/
│   │   ├── mod.rs
│   │   └── iced/
│   │       ├── mod.rs
│   │       ├── app.rs            # windows, input mapping, task scheduling
│   │       ├── ui_message.rs     # UI intentions and typed effect results
│   │       ├── view_model.rs     # control and message-feed projections
│   │       └── view.rs           # controls, Activity View and both windows
│   └── infrastructure/
│       ├── mod.rs
│       ├── samsung/
│       │   ├── mod.rs
│       │   ├── codec.rs          # bounded wire parsing and frame encoding
│       │   └── session.rs        # one socket owner, serial writes and TV events
│       ├── ssdp_discovery.rs     # bounded local-network candidate discovery
│       ├── preferences.rs       # non-secret saved-device and trust records
│       └── macos/
│           ├── mod.rs
│           └── keychain.rs       # device-scoped pairing-token store
├── tests/
│   ├── integration.rs
│   └── integration/             # public-boundary request and domain tests
├── assets/                       # bundled images, icons and macOS resources
└── docs/
    ├── architecture.md           # layer and runtime design
    ├── planned-repository-architecture.md
    ├── contribution-guide.md     # canonical development commands
    └── research/                 # evidence and hardware questions
```

## Module contracts

| Owner | Current responsibility | Dependencies |
| --- | --- | --- |
| `domain` | `DeviceId`, `DeviceDisplay`, and finite `RemoteAction` | Standard value types and Serde |
| `application` | `ControlState`, `SendRemoteAction`, `RemoteDispatcher`, `TvControlCoordinator`, and setup/discovery/session/storage ports | Domain and async runtime; no Iced, Samsung wire, or macOS APIs |
| `presentation::iced` | Window lifecycle, keyboard/focus mapping, Iced tasks, `RemoteControlViewState`, message feed, Activity projection | Application, domain, Iced |
| `infrastructure::samsung` | `SamsungGateway`, pinned TLS/WebSocket session, codec and `KEY_*` map | Application ports, domain, transport libraries |
| Other infrastructure | SSDP discovery, non-secret preferences/trust records, macOS Keychain | Application ports and platform libraries |
| `main.rs` | Construct concrete adapters and pass `AppServices` to Iced | Outer modules |

`TvControlCoordinator` owns restoration, discovery/probe attempts, Pairing and
reconnect flows, selection generations, session identity, cancellation, and
the sole bounded remote request FIFO. It exposes typed application results.
The Samsung adapter owns socket channels and frames. Iced schedules effects
and turns application outcomes into text; it does not import infrastructure.
The result journal retains each terminal outcome until Iced projects and
acknowledges it.

The request boundary is semantic: Iced submits `SendRemoteAction` with a
`RemoteAction::Enter` value, while the Samsung codec maps that action to
`KEY_ENTER`. A local socket write is a transport outcome, not observed TV
state. Sources, TV apps, text entry, wake, and exact volume remain future
features; their ports and modules should be added when behavior is ready.

## Runtime ownership

The app starts in Remote View and restores the selected saved TV if one is
readable. Settings handles discovery, manual address entry, identity/trust
confirmation, Pairing, re-pair, and forget. The coordinator validates
selection and attempt IDs before accepting asynchronous results. The Samsung
session serializes writes over secure port 8002. Iced owns the session-only
Global Messages Pane and Activity View projections. A written result reports
only that the frame reached the connection; no current TV-state readback is
claimed.

P1-M11 renamed the serialized action from `Select` to `Enter` without a legacy
reader. A pre-M11 `devices.json` containing `Select` is reported as corrupt
and left untouched. See the [manual recovery steps](phase-1/milestone-11-architecture.md#breaking-saved-action-spelling).

## Repository rules

- Keep the root for package metadata and top-level configuration. Put implementation under `src/`, integration tests under `tests/`, and bundled resources under `assets/`.
- Keep Samsung wire details within `infrastructure/samsung/`. Other TV transports can implement the same application port without changing ViewModels or use cases.
- Keep macOS APIs behind `infrastructure/macos/`. Packaging resources belong under `assets/` unless the selected bundle tool requires a small root-level configuration file.
- Plan for a self-signed, unnotarized macOS app bundle installed by the owner of the target Mac. A self-signed signature is not a Developer ID identity for Gatekeeper. Keep the bundle identifier and signing identity stable, and check local-network permission behavior after a rebuild. See [Apple's Developer ID guidance](https://developer.apple.com/developer-id/) and [local-network privacy guidance](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy).
- Add files when they acquire behavior; the tree is a responsibility map, not a requirement to create empty modules.
- Mirror important source boundaries under `tests/`. Use fake ports and synthetic frames; no automated test should require a TV or local network.
- Do not commit pairing tokens, real device identifiers, IP or MAC addresses, or local-network logs. Store secrets in Keychain and keep preferences separate.

The P1-M10 [sanitized hardware matrix](phase-1/milestone-10-hardware-matrix.md)
records tested support and deferred checks for KU75UA8090FXKR. Use the
[contribution guide](contribution-guide.md) for the validation workflow.
