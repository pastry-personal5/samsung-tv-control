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
│   │   ├── event.rs              # typed outcomes and sequenced control events
│   │   ├── state.rs              # pairing, connection and text-session states
│   │   ├── monitoring.rs         # snapshots, results and activity
│   │   ├── ports.rs              # transport, discovery, wake and storage traits
│   │   ├── dispatcher.rs         # sole bounded control queue and admission
│   │   ├── device_service.rs     # discovery, pairing, connect and forget
│   │   └── control_service.rs    # buttons, wake, sources, apps and text
│   ├── presentation/
│   │   ├── mod.rs
│   │   └── iced/
│   │       ├── mod.rs
│   │       ├── app.rs            # Iced application and main/settings window wiring
│   │       ├── message.rs        # UI and effect-result messages
│   │       ├── view_model.rs     # Remote and settings UI state projection
│   │       ├── messages.rs       # shared session-only Global Messages feed
│   │       ├── input.rs          # keyboard/pointer to command mapping
│   │       └── view.rs           # controls, Activity View and both windows
│   └── infrastructure/
│       ├── mod.rs
│       ├── samsung/
│       │   ├── mod.rs
│       │   ├── codec.rs          # bounded wire parsing and frame encoding
│       │   ├── session.rs        # one socket owner, serial writes and TV events
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
| `application` | `Command`, `ControlSnapshot`, `ControlEvent`, typed request results, connection/pairing/text states, use-case services, narrow ports such as `TvTransport`, `DeviceDiscovery`, `WakeSender`, `DeviceRepository`, `SecretStore`, and `TrustStore` | `domain` |
| `presentation::iced` | Iced `Message`, `ViewModel`, `view`, input mapper | `application`, `domain`, Iced |
| `infrastructure::samsung` | `TvTransport` implementation, codec and key map | `application` ports, `domain`, transport dependencies |
| Other `infrastructure` modules | Discovery, wake, persistence, Keychain and macOS network-access adapters | `application` ports, `domain`, platform dependencies |
| `main.rs` | Construction and lifecycle wiring | All outer modules |

Keep port traits near the use cases that need them. Define a small method set and typed errors for each port. A secret lookup should return only the credential needed for the selected device; ordinary device records must not serialize pairing tokens or certificate pins. The composition root injects concrete adapters into one application coordinator, then passes a command handle and read-only observation handle to the Iced adapter. `TvTransport` opens and closes a session, serially writes one typed operation handed to it, and reports transport results and TV observations. It has no command admission queue or product retry policy. The ViewModel updates presentation state and emits commands; `app.rs` submits fast controls to the ordered dispatcher and uses Iced tasks for longer operations. Admission returns a typed accepted or rejected result; each accepted request later has one terminal result. `monitoring.rs` retains unacknowledged terminal results in memory and supplies atomic snapshots, sequenced events, and a bounded Activity View projection to observers. `presentation::iced::messages` owns one app-wide, session-only feed of user-relevant messages from both windows, derived from typed outcomes and presentation events. Avoid importing Iced in `domain`, `application`, or `infrastructure`.

The command boundary is semantic. For example, the ViewModel emits `Command::Send { device, action: RemoteAction::Select }`; the Samsung key map chooses `KEY_ENTER`, and the codec creates the `ms.remote.control` frame. Exact volume setting, source selection, TV app launch, and text entry use separate command variants. App IDs must come from the selected TV's bounded catalog, with device membership checked again at dispatch. The session parses IME start/end events into typed observations; the application owns text state, and `text.rs` owns UTF-8 encoding into the wire format. UI events, application commands, Samsung frames, and Iced `Task` values are separate types with separate jobs. See [Iced application and subscription documentation](https://docs.rs/iced/0.14.0/iced/), [protocol research](research/samsung-tv-remote-protocol.md), [source and app research](research/source-changes-and-app-launches.md), and [text-input research](research/samsung-tv-text-input.md).

## Runtime ownership

`main.rs` builds the adapters and application services and starts the main Iced window in the Remote View. The application startup use case restores the most recently selected saved TV when one exists and initiates its Connection. Do not auto-open Settings or show onboarding on first launch. With no Selected TV, preserve the Remote View layout, disable controls, and show a short status line pointing to Settings. The Main Toolbar opens a separate Settings Window. Its TV settings page lists saved and discovered TVs in a radio-button TV Selection Table above Discover TVs, hiding the table when it has no rows. A single row is preselected; multiple rows require a choice when none is active. Selecting a discovered candidate requires TV Identity Confirmation and pairing before saving it as the active TV. The Iced adapter dispatches ViewModel commands in message order. The application coordinator owns the sole bounded command queue, connection policy, and sanitized monitoring state. One infrastructure session for the active TV owns the WebSocket and serializes writes handed to it. Fast controls enter the application queue synchronously without blocking the UI; Iced tasks await longer operations and map their results to presentation messages. Dropping or recreating the UI subscription must not duplicate the network connection. A newly attached observer receives a consistent state snapshot and subsequent sequenced events; a missed event triggers resynchronization. A terminal result is retained until acknowledged, or admission returns busy when its bounded journal is full.

Device discovery, wake retries, and connection attempts have explicit cancellation and time limits. The application layer decides when they stop; adapters implement the I/O. Request IDs and selection generations prevent results from a previous selection or connection from changing the current ViewModel. The UI renders states such as pairing pending, connected, reconnecting, re-pair required, text input available, and wake timeout without inferring success from a socket write alone. Only observed TV state with a known source and freshness can populate current power, source, mute, or numeric volume readings.

## Repository rules

- Keep the root for package metadata and top-level configuration. Put implementation under `src/`, integration tests under `tests/`, and bundled resources under `assets/`.
- Keep Samsung wire details within `infrastructure/samsung/`. Other TV transports can implement the same application port without changing ViewModels or use cases.
- Keep macOS APIs behind `infrastructure/macos/`. Packaging resources belong under `assets/` unless the selected bundle tool requires a small root-level configuration file.
- Plan for a self-signed, unnotarized macOS app bundle installed by the owner of the target Mac. A self-signed signature is not a Developer ID identity for Gatekeeper. Keep the bundle identifier and signing identity stable, and check local-network permission behavior after a rebuild. See [Apple's Developer ID guidance](https://developer.apple.com/developer-id/) and [local-network privacy guidance](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy).
- Add files when they acquire behavior; the tree is a responsibility map, not a requirement to create empty modules.
- Mirror important source boundaries under `tests/`. Use fake ports and synthetic frames; no automated test should require a TV or local network.
- Do not commit pairing tokens, real device identifiers, IP or MAC addresses, or local-network logs. Store secrets in Keychain and keep preferences separate.

The current `Cargo.toml` and `src/main.rs` are a minimal starting point. Dependencies and bundle configuration should be selected when their implementation is ready to be validated. The Settings Window's TV page is the entry point for network discovery and manual host entry. Discovery presents candidates for user selection and saves a TV only after TV Identity Confirmation and required pairing. At launch, restore the most recently selected saved TV and attempt its Connection using the normal reconnect policy; manual host entry remains available if discovery is unavailable or finds nothing. Select the discovery mechanism after testing the TV's advertisements. KU75UA8090FXKR is the initial hardware target, not yet a verified support claim. The port-8001 compatibility policy remains open until that TV is tested. Use the commands in the [contribution guide](contribution-guide.md) for the repository's validation workflow.
