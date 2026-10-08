# Samsung TV Remote

Samsung TV Remote is a super-fast remote controller for macOS, written in Rust.

It is intended to provide responsive, keyboard-friendly control of compatible Samsung TVs on the local network.

## Status

The repository has a native Iced app with Remote, Sources, Apps, Text Input,
and Settings views. P1-M10 secure TV setup and control are in progress: SSDP
discovery and manual address entry, certificate confirmation, Keychain-backed
pairing, saved-TV reconnect, and bounded remote dispatch are implemented.
The owner verified empty-install setup, Pairing, saved-token reconnect,
discovery, and all ten current key actions from a signed app bundle.
Discovery's empty/denied/timeout manual-entry fallback, local-network
permission recovery, and failure/switching paths still need human verification.
Those checks, the remaining native-bundle review, and optional latency
measurement are deferred without a revisit date. The latency target has not
been measured.

## Technology and architecture

The [software architecture](docs/architecture.md) defines the selected UI and
runtime design. The [planned repository architecture](docs/planned-repository-architecture.md)
maps it to module responsibilities. The [control and monitoring review](docs/control-monitoring-review.md)
records the design findings and implementation checks.

The current UX plan defines [canonical terms](docs/ux-term.md), the
[information architecture](docs/planned-information-architecture.md), and the
[planned GUI](docs/ux-gui.md). These documents guide the current shell and
later device behavior.

## Development

See the [contribution guide](docs/contribution-guide.md) for the Cargo
workflow, validation gates, and pull-request expectations. The
[development process](docs/development-process.md) defines phase and milestone
planning. [AGENTS.md](AGENTS.md) covers repository layout, code style, and
testing guidance.

Do not commit Samsung pairing tokens, device IP addresses, or other
local-network details.
