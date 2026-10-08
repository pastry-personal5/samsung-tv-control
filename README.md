# Samsung TV Remote

Samsung TV Remote is a super-fast remote controller for macOS, written in Rust.

It is intended to provide responsive, keyboard-friendly control of compatible Samsung TVs on the local network.

## Status

The repository has a native Iced app with Remote, Sources, Apps, Text Input,
and Settings views. P1-M10 and P1-M11 are Done. SSDP discovery, manual address
entry, certificate confirmation, Keychain-backed Pairing, saved-TV reconnect,
and bounded remote dispatch are implemented. The application coordinator owns
the live workflow; Iced handles input, windows, and display.
The owner verified empty-install setup, Pairing, saved-token reconnect,
discovery, and all ten current key actions from a signed app bundle.
Discovery's empty/denied/timeout manual-entry fallback, local-network
permission recovery, and failure/switching paths still need human verification.
The owner closed P1-M10 with those checks, the remaining native-bundle review,
and latency measurement deferred without a revisit date. The latency target
has not been measured.

P1-M11 renamed the stored Directional Pad center action from `Select` to
`Enter`. Older `devices.json` files containing `Select` cannot be read by this
build. To set up the TV again, quit the app, move
`~/Library/Application Support/Samsung TV Remote/devices.json` to a safe
backup location outside that directory, relaunch, and pair the TV. The app
does not delete the old file, trust records, or Keychain items automatically.

## Technology and architecture

The [software architecture](docs/architecture.md) defines the selected UI and
runtime design. The [repository architecture](docs/planned-repository-architecture.md)
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
