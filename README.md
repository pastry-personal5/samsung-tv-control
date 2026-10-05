# Samsung TV Remote

Samsung TV Remote is a responsive remote controller for macOS, written in Rust.

It is intended to provide responsive, keyboard-friendly control of compatible Samsung TVs on the local network.

## Status

The repository has a minimal Rust crate. The macOS interface and device
protocol implementation have not been built yet.

## Technology and architecture

The [software architecture](docs/architecture.md) defines the selected UI and
runtime design. The [planned repository architecture](docs/planned-repository-architecture.md)
maps it to module responsibilities.

The current UX plan defines [canonical terms](docs/ux-term.md), the
[information architecture](docs/planned-information-architecture.md), and the
[planned GUI](docs/ux-gui.md). These are design documents; the GUI has not yet
been implemented.

## Development

See the [contribution guide](docs/contribution-guide.md) for the Cargo
workflow, validation gates, and pull-request expectations. The
[development process](docs/development-process.md) defines phase and milestone
planning. [AGENTS.md](AGENTS.md) covers repository layout, code style, and
testing guidance.

Do not commit Samsung pairing tokens, device IP addresses, or other
local-network details.
