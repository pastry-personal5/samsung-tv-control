# P1-M13: Wake-on-LAN and Toggle Power

Status: Implemented; live-TV Wake verification deferred

## Contracts and ownership

Add a domain value for a normalized six-octet MAC address (six raw octets with
canonical display formatting) and an application `WakeConfiguration` containing
optional wired/Wi-Fi addresses and an active interface choice. Parsing accepts
only exactly six hexadecimal octets with the documented separators; it rejects
multicast, all-zero, and broadcast MACs. An active choice is valid only when
its corresponding MAC is present. Store this non-secret configuration in
`SavedDevice` and preferences; do not put it in Keychain, logs, messages,
fixtures, or activity text.

Extend the device disk record with optional wake fields that deserialize as
absent for existing records. The preferences reader must accept the current
pre-wake record version and the new version, then write only the new version
after a successful save. A malformed wake field is retained as a
configuration-required state for that same device rather than corrupting the
whole device repository or silently selecting a different device. An absent
field is likewise a valid configuration-required state. This migration is
required even though the values are non-secret, because saved paired devices
must remain usable.

The application declares a narrow asynchronous wake transport port that accepts
the selected TV host and active MAC and returns a typed local-send result. The
infrastructure adapter resolves and re-validates the host using the existing
local-address policy, selects a reachable IPv4 address and its egress
interface, and derives that interface's directed broadcast from its IPv4
address and netmask. It rejects no IPv4 answer, no local route, /31 or /32
routes, and route/interface lookup failures with typed, user-safe errors. It
must not substitute 255.255.255.255, a stale cached route, or a different
interface. The adapter owns interface binding where required, UDP socket
options, UDP/9, and the exact packet bytes. It enables broadcast and sends the
102-byte packet once. A successful `send_to` is recorded only as “magic packet
sent”, never “TV is on”.

`TvControlCoordinator` owns wake generation and exposes typed wake snapshots:
idle/configuration-required, packet-sending, packet-sent, reconnecting,
ready, failed, and cancelled. `begin_wake` synchronously validates selection,
configuration, and disconnected state, increments a wake attempt, and snapshots
the device ID, selection generation, and configuration before it starts work.
It begins immediately with the one transport attempt, then schedules bounded
trusted reconnect/readiness attempts until the paired remote session is usable
or a monotonic 30-second deadline expires. A retry cadence must be bounded and
must check cancellation before each connection attempt; it is not a wake-packet
retry. The first usable session is installed through the same guarded session
path as normal reconnect before the snapshot becomes `ready`.

Selecting, forgetting, re-pairing, editing wake configuration, and an explicit
cancel invalidate the wake attempt and cancel pending readiness work. Every
transport or reconnect completion must match both the attempt and selection
generation before it mutates state or installs a session. Cancellation cannot
retract a datagram already handed to the OS. The state distinguishes pre-send
cancellation, unknown send outcome, and post-send reconnect cancellation; it
prevents any subsequently scheduled wake packet. No retry path may invoke the
wake transport after the first attempt.

Presentation adds `PrimaryView::Power` immediately above Remote in the Sidebar.
Power View owns all Wake Steps and wake controls, including retry/cancel and a
Power Toggle control; Remote View retains its existing Power Toggle button but
does not duplicate Wake Steps. The view model projects the same coordinator
wake snapshot into Power View whether Wake starts there or through Remote View.
The Remote button emits one typed power intent, and the application coordinator
decides whether it is eligible for `KEY_POWER` or starts a trusted connection
attempt with an approximately two-second monotonic deadline, followed by Wake
on an unreachable-TV failure. The short attempt must use saved pairing material
and certificate trust; a plain TCP port check is not trusted readiness. Pairing
denial, rejected or missing credentials, changed trust, and storage errors stop
the flow with recovery guidance instead of causing a wake packet. On the disconnected path,
the presentation selects Power View in the same update that starts the attempt,
so the Sidebar selection and visible progress change together. A failed or
stale intent cannot navigate to an unrelated TV's progress. Power Toggle is
disabled while the short connection check or wake flow is active; Cancel
remains available in Power View.

Presentation maps typed facts into a persistent Wake Steps component. Rows are
non-interactive radio-style status indicators labeled Wake configuration,
Magic packet, Reconnecting, and Remote ready. Each row has a textual state;
color and the active row's bounded UI-tick-driven blink are supplementary.
The component's retry control emits new wake intent only after terminal failure
or cancellation, is disabled while an attempt is active, and names the local
failure class without exposing host or MAC values. Wake is disabled for absent
configuration and existing live connection; transient local-send failures stay
actionable through Try again. Power Toggle sends `KEY_POWER` only for a live
connection. While disconnected, it starts the short trusted connection check
only when an active Wake MAC is configured.

Samsung's codec becomes the sole `KEY_POWER` owner. It maps semantic
`PowerToggle` only after application policy admits the connected path; it still owns frame
encoding and write uncertainty. Update `ControlState` policy so Power Toggle
has the same selected-target, generation, pairing, and `ConnectionState::Ready`
checks as the live remote actions before a `KEY_POWER` write. The disconnected
path is a separate wake intent with its own configuration and lifecycle checks;
it never enters the remote-action queue. Remove the deferred-action exception.
The dispatcher must recheck those conditions immediately before the write, so a
queued toggle cannot reach a session that disconnected after the button was
pressed.

## Tests

- MAC normalization and invalid input (including multicast, zero, and broadcast
  values); active-interface validation; persistence round trips; loading of
  pre-wake records; and same-device recovery from malformed wake fields.
- Fake wake transport verifies exact six-`FF` plus 16-MAC magic-packet shape,
  selected active MAC, UDP/9, one immediate send, typed local-send failure,
  and no global-broadcast or wrong-interface fallback. Route tests cover IPv4
  selection, IPv6-only resolution, and unusable-prefix rejection.
- Coordinator scenarios cover ready after reconnect, 30-second timeout,
  cancellation before send and during reconnect, Try again, selection
  replacement, configuration edit, forget, stale effects, and installing the
  first ready session only once.
- Presentation tests cover disabled/configured/connected Wake states, each
  Wake Steps state, non-color text, active blink projection, Sidebar order,
  Power View selection, and automatic navigation on the disconnected Remote
  power path. Verify the short probe's deadline and that missing Wake configuration
  disables Remote Power Toggle only while disconnected.
- Codec and policy tests cover connected-only Power Toggle and `KEY_POWER`.
  Dispatcher tests cover a queued Power Toggle becoming ineligible before its
  write starts.
