# UX GUI specification

Status: Active

This records the current P1-M14 interface and owner decisions. Names are defined
in [UX terms](ux-term.md); placement is defined in the
[information architecture](ux-information-architecture.md). The
[research note](research/discovery-pairing-wake-ux.md) contains discovery,
pairing, and Wake proposals that are not yet approved or implemented.

## Window and visual system

The app uses an always-dark graphite theme, blue interaction accents, and
semantic status colors with text equivalents. The main window minimum is 1100 ×
760; the Settings Window minimum is 1000 × 660.

The main Sidebar is a compact icon rail with Power above Remote, then Sources,
Apps, and Text Input; Settings stays at the bottom. Each button slot fills the
rail width with no outer side margin or vertical gap. Original local bitmap
icons provide the faces. Resting, hover, selected, focus, and disabled states
should be distinct. Icon destinations keep view-name tooltips and accessible
names, subject to native accessibility verification.

The Main Pane shows one horizontally centered, bounded-width box titled with the
current primary view's name. Its primary-view region scrolls rather than
overlapping the Global Messages Pane. Power View groups Wake Steps and its
actions in the box. Sources, Apps, and Text Input use titled placeholder boxes;
their capabilities are not implemented.

## Remote View

Remote View contains only the bundled visual remote image. Clickable regions
cover Power Toggle; Up, Left, Enter, Right, Down; Back, Home, Play/Pause; Mute;
and Volume Up and Down. Usable regions highlight on pointer hover; unavailable
regions are dimmed. There is no companion “Keyboard controls” heading or panel
and no second row of action buttons below the image.

Mapped shortcuts continue to invoke their actions while Remote View is active.
The image's pointer regions have no individual keyboard focus, tooltip, or
screen-reader description in this milestone. The native accessibility tree
currently does not expose the individual controls; complete screen-reader and
focus review remains open. Do not claim accessible control names are verified
until that review passes.

Play/Pause sends Samsung KEY_PLAY for now. Its pause behavior on the target TV
has not been verified, and the interface must not claim a confirmed toggle.
There is no exact-volume slider.

With a live paired session, Power Toggle sends the TV's power-toggle key and
stays in Remote View. When disconnected, the Remote View power region opens
Power View and starts a brief trusted connection check. If the TV is unreachable
and a valid active Wake MAC exists, it proceeds to Wake. A pairing failure stops
before sending a packet. The disconnected region is unavailable without Wake
configuration.

## Power View

Wake Steps remain visible and name the current phase: configuration, connection
check, packet, reconnection, and paired remote readiness. Wake sends one magic
packet to the configured active MAC, then waits up to 30 seconds for a paired
remote channel. It does not repeat the packet. A sent packet, a ready remote
channel, and visible TV panel power are different outcomes; the app cannot
measure the last one.

Power Toggle sends Samsung KEY_POWER only over a live paired connection. Try
again and Cancel appear when their flow permits them. Cancellation may stop
waiting but cannot retract a packet already sent. Controls and step text keep
visible names and text equivalents for status colors.

## Discovery and pairing in Settings

The Settings Sidebar has Discovery and Wake on LAN. The Discovery page groups TV
List, Discovery and Pairing, Connection recovery, and Guidance cards.

TV List shows Saved TVs with a selection radio. The selected Saved TV row offers
Check TV; Re-pair becomes available after a matching check. Unsaved discovered
or manually entered hosts appear in a separate candidate list. Selecting a
candidate stages its address without selecting it as the app's trusted TV. Check
TV probes its secure endpoint on port 8002 and displays the observed certificate
SHA-256 plus name and model when available. Pair becomes available only for the
checked host. Guidance asks the user to confirm the intended TV; pairing needs
approval on the physical TV. Only a successful pair saves, selects, and connects
that TV. A changed host must be checked again.

Discover TVs starts a local scan. Manual IP address or host-name entry remains
available, including after an empty, failed, or permission-denied scan;
submitting the field checks that host. Guidance describes search, probe,
pairing, and recovery results. Saved TVs remain available during a failed scan.
Retry Connection and Forget Selected TV belong to the Connection recovery card.

This page currently uses one guidance/status region; it does not provide a
separate scan Cancel button, a per-row pairing progress stepper, or a dedicated
identity-confirmation dialog. Do not describe those as implemented.

## Wake on LAN in Settings

The Wake on LAN page has wired and Wi-Fi MAC fields, an active choice of Wired,
Wi-Fi, or Disabled, and Guidance. A valid active choice with a Selected TV is
saved automatically. A valid draft entered before pairing a new TV can be
applied when pairing succeeds. The fields must refer to the MAC shown by the
intended TV for the chosen network interface; a host IP address is not the MAC.
Invalid or incomplete entries are not a usable active Wake target.

Wake configuration does not prove that the TV supports waking from its current
standby state or that the network will deliver the packet. The Power View
presents the actual attempt and remote-readiness result.

## Feedback and accessibility

The Global Messages Pane stays below every primary view. The pointer-draggable
Messages split bar adjusts its height while preserving usable room for the
primary view. The feed is scrollable, chronological, and session-only, newest at
the bottom. Rows have fixed-width Info or Warning severity labels and concise
message text. It excludes sequence numbers, source bookkeeping, raw network
payloads, credentials, and entered text. Routine “Opened Settings” and “Closed
Settings” messages are omitted. Setup status can also appear locally in
Settings.

Report “sent” or “requested” for remote commands unless the TV reports a
resulting state. Show pairing, connection, and Wake failures with the next
available action. Keep local packet-send success distinct from remote readiness.
Disabled native controls should explain their known reason; the Remote View
image currently gives dimming without per-region spoken guidance. Native
keyboard-focus and screen-reader behavior remains a P1-M14 review item.
