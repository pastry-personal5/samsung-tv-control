# P1-M13: Wake-on-LAN and Toggle Power

Status: Active

Implementation complete; live-TV Wake verification deferred.

## Goal

Provide honest, observable wake of a selected Samsung TV in a dedicated Power
View, while retaining Power Toggle in Remote View. A Power Toggle press for a
saved TV without a live remote connection opens Power View and attempts a
short (about two seconds) trusted reconnect first. If the TV is unreachable,
the flow starts Wake when its configuration permits.

## User journey

1. In Settings > Wake on LAN, the user enters optional wired and Wi-Fi MAC
   addresses, then chooses one configured address as the active wake interface.
   The setting is saved before it is reported as usable.
2. The Sidebar places Power immediately above Remote. Selecting Power opens
   Power View with Wake Steps, a dedicated Wake control, and Power Toggle.
   Without a valid active MAC, Wake is disabled with an explanation.
3. Pressing Wake sends exactly one immediate magic packet. The UI records this
   as an attempted packet send, not confirmation that the TV woke.
4. The app retries only trusted remote readiness for up to 30 seconds. It
   completes Wake only when the paired remote channel is usable.
5. Wake Steps marks completed rows green, its active row blinking, and a
   failure row red with textual guidance and a Try again action.
6. Remote View keeps its Power Toggle button. When the selected TV has no live
   remote connection, pressing it selects Power in the Sidebar, displays Power
   View, and tries connecting before Wake. If pairing or stored credentials need
   repair, it stops with guidance. A live paired remote session sends
   the Samsung power toggle command and stays in Remote View.

## Scope

- Persist optional wired and Wi-Fi MACs plus a user-selected active interface
  with each saved TV. MAC input is normalized before save; malformed input is
  rejected rather than persisted. These are non-secret local preferences.
- Send a standard 102-byte magic packet for the active MAC to the selected
  TV's validated IPv4 local-route broadcast address on UDP port 9. If the
  saved host resolves only to IPv6, has no local IPv4 route, or the route has
  no usable broadcast address, Wake fails locally with guidance; it never
  falls back to an arbitrary interface or global broadcast.
- Add a dedicated Wake workflow, progress model, cancellation/stale-result
  protection, bounded readiness retries, and Global Messages facts.
- Add a Power Sidebar destination and Power View. Keep Remote View's Power
  Toggle; route its intent according to the selected TV's live connection
  state. Map the connected path to Samsung `KEY_POWER`.

## Out of scope

- Sending to both MACs, automatic MAC discovery, cloud/SmartThings control,
  Consumer IP Control `powerOn`, panel-state detection, and a claim that a
  UDP send or socket write proves physical TV state.
- Repeating WoL packets after the initial send. Late packets can cause an
  unwanted wake and are prohibited.

## Completion checklist

- [x] Valid MAC configuration survives restart, including loading existing
  device records that predate Wake configuration; missing/incomplete active
  configuration disables Wake with an explanatory tooltip.
- [x] Wake sends one immediate packet, retries readiness for no longer than 30
  seconds, and completes only when the remote channel is usable.
- [x] Wake cancellation, selection replacement, forget, and stale completions
  cannot send a later packet or change the new TV's progress.
- [x] Power Toggle is encoded only for an active connected session.
- [x] Power is above Remote in the Sidebar; both manual Wake and the Remote
  View's disconnected Power Toggle show Power View progress. The Remote path
  attempts a short trusted connection before sending a wake packet; without a
  configured active MAC, its Power Toggle button stays disabled while
  disconnected. A second press is disabled until the flow finishes.
- [x] Wake states remain understandable without color or animation.
- [x] Deterministic tests and documented Cargo gates pass; hardware evidence,
  if gathered, is sanitized and does not claim unmeasured wake latency.
- [ ] Physical Wake, macOS Local Network permission recovery, and time to a
  usable remote connection receive hardware acceptance.

Live-TV Wake, macOS Local Network permission recovery, and measured time to
usable remote connection have not been verified on hardware. A successful UDP
send proves only that the packet was handed to the local network stack.

## References

- [Architecture](milestone-13-architecture.md)
- [Wake-on-LAN research](../archive/research/samsung-tv-wake-on-lan.md)
