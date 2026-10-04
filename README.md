# Samsung TV Remote

Samsung TV Remote is a super-fast remote controller for macOS, written in Rust.

It is intended to provide responsive, keyboard-friendly control of compatible Samsung TVs on the local network.

## Status

The repository is being initialized. The Rust crate, macOS interface, and
device protocol implementation have not been committed yet.

## Technology and architecture

The [architecture](docs/architecture.md) defines the selected technologies, software architecture.

## Development

See the [contribution guide](docs/contribution-guide.md) for the Cargo
workflow, validation gates, and pull-request expectations. The
[development process](docs/development-process.md) defines phase and milestone
planning. [AGENTS.md](AGENTS.md) covers repository layout, code style, and
testing guidance.

Do not commit Samsung pairing tokens, device IP addresses, or other
local-network details.
