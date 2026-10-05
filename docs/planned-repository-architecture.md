# Planned repository architecture

This is the proposed layout for the Rust macOS application. It maps the [software architecture](architecture.md) to directories and files. These paths describe ownership; most do not exist yet. Start with one Cargo package and add files as their behavior is implemented.

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
│   │   ├── device.rs             # stable local identity and device record types
│   │   ├── capability.rs         # known, unknown and unsupported capabilities
│   │   ├── remote_action.rs      # finite semantic action set
│   │   └── target.rs             # typed sources and installed TV app references
│   ├── application/
│   │   ├── mod.rs
│   │   ├── command.rs            # typed requests from all input surfaces
│   │   ├── event.rs              # typed outcomes and session events
│   │   ├── state.rs              # pairing, connection and text-session states
│   │   ├── ports.rs              # session, discovery, wake and storage traits
│   │   ├── dispatcher.rs         # sequenced, bounded command admission
│   │   ├── device_service.rs     # discovery, pairing, connect and forget
│   │   └── control_service.rs    # buttons, wake, sources, apps and text
│   ├── presentation/
│   │   ├── mod.rs
│   │   └── iced/
│   │       ├── mod.rs
│   │       ├── app.rs            # Iced application, Task and Subscription wiring
│   │       ├── message.rs        # UI and effect-result messages
│   │       ├── view_model.rs     # UI state projection and update logic
│   │       ├── input.rs          # keyboard/pointer to command mapping
│   │       └── view.rs           # widgets and layout
│   └── infrastructure/
│       ├── mod.rs
│       ├── samsung/
│       │   ├── mod.rs
│       │   ├── codec.rs          # bounded wire parsing and frame encoding
│       │   ├── session.rs        # one socket owner and serialized send queue
│       │   ├── keys.rs           # model-aware action to remote-key mapping
│       │   ├── apps.rs           # installed-app events and launch frames
│       │   └── text.rs           # IME events and text-entry frames
│       ├── probe.rs              # manual-host validation and endpoint probing
│       ├── discovery.rs          # bounded local-network candidate discovery
│       ├── wake.rs               # Wake-on-LAN packet and readiness adapter
│       ├── preferences.rs        # non-secret local device records
│       └── macos/
│           ├── mod.rs
│           ├── keychain.rs       # device-scoped pairing-token store
│           ├── trust.rs          # device-scoped certificate trust records
│           └── local_network.rs  # platform access-error interpretation
├── tests/
│   ├── application/              # fake-port command and state scenarios
│   ├── infrastructure/           # synthetic Samsung protocol fixtures
│   └── presentation/             # input mapping and ViewModel behavior
├── assets/                       # bundled images, icons and macOS resources
└── docs/
    ├── architecture.md           # layer and runtime design
    ├── planned-repository-architecture.md
    ├── contribution-guide.md     # canonical development commands
    └── research/                 # evidence and hardware questions
```

## Module contracts

| Owner | Public surface | Dependency direction |
| --- | --- | --- |
| `domain` | `DeviceId`, `Device`, `RemoteAction`, typed source/app references and capability values | Standard library and small value-type dependencies only |
| `application` | `Command`, `Event`, connection/pairing/text states, use-case services, narrow ports such as `RemoteSession`, `DeviceDiscovery`, `WakeSender`, `DeviceRepository`, `SecretStore`, and `TrustStore` | `domain` |
| `presentation::iced` | Iced `Message`, `ViewModel`, `view`, input mapper | `application`, `domain`, Iced |
| `infrastructure::samsung` | `RemoteSession` implementation, codec and key map | `application` ports, `domain`, transport dependencies |
| Other `infrastructure` modules | Discovery, wake, persistence, Keychain and macOS network-access adapters | `application` ports, `domain`, platform dependencies |
| `main.rs` | Construction and lifecycle wiring | All outer modules |

Keep port traits near the use cases that need them. Define a small method set and typed errors for each port. A secret lookup should return only the credential needed for the selected device; ordinary device records must not serialize pairing tokens or certificate pins. The composition root injects concrete adapters and one shared session handle into the application services, then passes cloneable service handles to the Iced adapter. The ViewModel updates presentation state and emits commands; `app.rs` submits fast controls to the ordered dispatcher and uses Iced tasks for longer operations. Command admission assigns a sequence and returns accepted or busy without waiting for the TV. Completion reports written or uncertain separately. Avoid importing Iced in `domain`, `application`, or `infrastructure`.

The command boundary is semantic. For example, the ViewModel emits `Command::Send { device, action: RemoteAction::Select }`; the Samsung key map chooses `KEY_ENTER`, and the codec creates the `ms.remote.control` frame. Source selection, TV app launch, and text entry use separate command variants. App IDs must come from the selected TV's bounded catalog, with device membership checked again at dispatch. The session handles IME start/end events and exposes text state; `text.rs` owns UTF-8 encoding into the wire format. UI events, application commands, Samsung frames, and Iced `Task` values are separate types with separate jobs. See [Iced application and subscription documentation](https://docs.rs/iced/0.14.0/iced/), [protocol research](research/samsung-tv-remote-protocol.md), [source and app research](research/source-changes-and-app-launches.md), and [text-input research](research/samsung-tv-text-input.md).

## Runtime ownership

`main.rs` builds the adapters and application services, then starts an Iced regular window. The Iced adapter dispatches ViewModel commands in message order. The services own connection policy and expose sanitized events. One session owner for the active TV owns the WebSocket, serializes writes through a bounded queue, and emits events; the Iced subscription observes that stream. Fast controls enter the queue synchronously without blocking the UI; Iced tasks await longer operations and map their results to presentation messages. Dropping or recreating the UI subscription must not duplicate the network connection. A newly attached observer receives a consistent state snapshot and subsequent sequenced events; a missed event triggers resynchronization.

Device discovery, wake retries, and connection attempts have explicit cancellation and time limits. The application layer decides when they stop; adapters implement the I/O. Operation IDs prevent results from a previous selection or connection from changing the current ViewModel. The UI renders states such as pairing pending, connected, reconnecting, re-pair required, text input available, and wake timeout without inferring success from a socket write alone.

## Repository rules

- Keep the root for package metadata and top-level configuration. Put implementation under `src/`, integration tests under `tests/`, and bundled resources under `assets/`.
- Keep Samsung wire details within `infrastructure/samsung/`. Other TV transports can implement the same application port without changing ViewModels or use cases.
- Keep macOS APIs behind `infrastructure/macos/`. Packaging resources belong under `assets/` unless the selected bundle tool requires a small root-level configuration file.
- Plan for a self-signed, unnotarized macOS app bundle installed by the owner of the target Mac. A self-signed signature is not a Developer ID identity for Gatekeeper. Keep the bundle identifier and signing identity stable, and check local-network permission behavior after a rebuild. See [Apple's Developer ID guidance](https://developer.apple.com/developer-id/) and [local-network privacy guidance](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy).
- Add files when they acquire behavior; the tree is a responsibility map, not a requirement to create empty modules.
- Mirror important source boundaries under `tests/`. Use fake ports and synthetic frames; no automated test should require a TV or local network.
- Do not commit pairing tokens, real device identifiers, IP or MAC addresses, or local-network logs. Store secrets in Keychain and keep preferences separate.

The current `Cargo.toml` and `src/main.rs` are a minimal starting point. Dependencies and bundle configuration should be selected when their implementation is ready to be validated. The add-device flow includes network discovery and manual host entry; the discovery mechanism should be selected after testing the TV's advertisements. KU75UA8090FXKR is the initial hardware target, not yet a verified support claim. The port-8001 compatibility policy remains open until that TV is tested. Use the commands in the [contribution guide](contribution-guide.md) for the repository's validation workflow.
