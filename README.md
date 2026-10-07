# Samsung TV Remote

Samsung TV Remote is a super-fast remote controller for macOS, written in Rust.

It is intended to provide responsive, keyboard-friendly control of compatible Samsung TVs on the local network.

## Status

The repository has a native Iced GUI shell with Remote, Sources, Apps, Text
Input, and Settings views. Device discovery, pairing, connection, and remote
commands are not implemented yet.

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
