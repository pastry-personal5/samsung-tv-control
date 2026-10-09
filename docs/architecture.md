# Software architecture

Status: Active

Samsung TV Remote is one Rust package. Its dependency direction is
`presentation → application → domain` and `infrastructure → application/domain`.
`main.rs` wires the adapters. UI code never constructs Samsung wire frames or
owns a socket; application policy never imports Iced or macOS APIs.

| Area | Responsibility |
| --- | --- |
| `src/domain/` | Device identity, MAC address, and finite semantic remote actions. |
| `src/application/` | TV selection, trust, pairing, connection, command admission and dispatch, Wake flow, typed outcomes, and ports. |
| `src/infrastructure/` | Secure Samsung session and codec, SSDP discovery, UDP Wake, preferences, and macOS Keychain. |
| `src/presentation/iced/` | Windows, input mapping, visual remote, view state, and user-facing feedback. |
| `src/main.rs` | Composition of concrete adapters and the app. |

The application coordinator owns the Selected TV, selection generation,
connection lifecycle, and one bounded command queue. Each request targets a
specific saved device and returns a typed rejection or transport outcome.
The Samsung session serializes writes on the secure port-8002 WebSocket.
A successful socket write means a command was sent; it does not prove that
power, volume, or another TV state changed. Do not replay uncertain actions
automatically.

Discovery and manual entry yield untrusted candidates. Check TV probes the
secure endpoint and presents its observed certificate identity. Pairing
requires confirmation of the intended TV and approval on that TV. Pairing
tokens belong in Keychain; device preferences and certificate trust are stored
separately. A changed certificate fails the saved trust check before sending a
token. Selection, pairing, and a live connection are separate facts.

Wake uses a selected TV's configured wired or Wi-Fi MAC address. It sends one
local-route magic packet, then waits for a usable paired remote connection.
Packet transmission does not prove delivery or visible panel power. A live
paired Power Toggle sends `KEY_POWER`; the visual remote's Play/Pause currently
sends `KEY_PLAY`, with pause behavior awaiting hardware verification.

The Iced presentation maps pointer and keyboard input to typed requests and
projects application results into the current [UX specification](ux-gui.md).
Sources, Apps, and Text Input remain placeholders. Native keyboard focus and
screen-reader support need further verification; see [Phase 1](phase-1/phase-1.md).

Use deterministic fakes for protocol, pairing, and network-boundary tests.
The [contribution guide](contribution-guide.md) defines validation. Historical
designs and milestone plans are in the [archive](archive/).
