# Planned GUI

Status: Archived

This document describes the first planned GUI for the macOS app. It is a
behavior and hierarchy specification, not a pixel mockup. See the canonical
[UX terms](initial-ux-term.md) and [planned information architecture](planned-information-architecture.md).

## Window structure

The main app window has two main parts:

- **Sidebar** on the left. It contains **Power**, **Remote**, **Sources**, **Apps**, and
  **Text Input**, so the user can return to the startup view after navigating.
  The icon-only **Settings button** is anchored at the bottom of the Sidebar.
- **Main Pane** on the right. It opens to the **Remote View**. Clicking Power,
  Sources, Apps, or Text Input replaces the Main Pane's primary view with
  **Power View**, **Sources View**, **Apps View**, or **Text Input View** respectively.

The **Settings button** opens a separate **Settings Window**. That window has
its own **Settings Sidebar** and **Settings Main Pane**. The first Settings
Sidebar item is **TV**. Selecting it displays TV settings in the Settings Main
Pane. The Settings Window's other sidebar items and its window modality are
TBD.

```text
┌─────────────────────┬─────────────────────────────────────┐
│ Sidebar             │ Main Pane: Remote View              │
│ Power               │                                     │
│ Remote              │                                     │
│ Sources             │                                     │
│ Apps                │                                     │
│ Text Input          │                                     │
│                     │              Power Toggle           │
│                     │                                     │
│                     │                  Up                 │
│                     │          Left  Enter  Right         │
│                     │                 Down                │
│                     │                                     │
│                     │           Back       Home           │
│                     │                                     │
│                     │          Volume Slider              │
│                     │      Mute   Volume Down   Volume Up  │
│                     ├─────────────────────────────────────┤
│                     │ ↕ Messages split bar                │
│                     ├─────────────────────────────────────┤
│                     │ Global Messages Pane                │
│                     │ Info    concise user message        │
│                     │ Warning concise user message        │
└─────────────────────┴─────────────────────────────────────┘

Settings Window
┌─────────────────────┬─────────────────────────────────────┐
│ Settings Sidebar    │ Settings Main Pane: TV settings     │
│ TV                  │                                     │
│ other items TBD     │ TV Selection Table (when non-empty) │
│                     │ (o) TV name     Saved               │
│                     │ ( ) TV name     Discovered          │
│                     │                                     │
│                     │              [Discover TVs]         │
└─────────────────────┴─────────────────────────────────────┘
```

The sketch communicates order and grouping only; the P1-M14 visual decisions
below supersede its text-button depiction. A draggable split bar
between the primary view and Global Messages Pane adjusts the latter's height.
The Sidebar items replace only the primary view; the Global Messages Pane
remains in the Main Pane below it.

## Main Pane navigation

**Power** opens Power View. **Remote** returns to Remote View. **Sources** opens Sources View, **Apps**
opens Apps View, and **Text Input** opens Text Input View. Each view targets the
current Selected TV and uses the
same Pairing, Connection, capability, request feedback, and Global Messages
Pane as Remote View. Changing views does not switch TVs or open a second
Connection. Keep the Settings button available from all views.

Sources View presents supported source choices from the Selected TV and the
TV's source chooser when direct choices are unavailable. Report a source
change as requested unless the TV confirms the new source. Apps View presents
the Selected TV's reported installed apps and refresh status; it must distinguish
an empty catalog from loading, unavailable, or stale data. Report an app launch
as requested until the TV provides stronger evidence. Text Input View shows
whether TV text input is available or requires the user to focus a TV field,
provides a draft and Send action when allowed, and preserves uncertain text in
memory with an explicit warning. It must never persist or log entered text.

## Power View and Remote View hierarchy

Do not show a persistent Selected TV, Pairing, Connection, or aggregate
availability status region above the controls. Disabled controls provide a
specific accessible reason and tooltip; user-relevant outcomes appear in the
Global Messages Pane. This must not shift the control order.

Power View places persistent **Wake Steps** above a power-control group with
**Wake** and **Power Toggle**. Wake Steps shows configuration, magic-packet,
reconnection, and remote-ready rows. Its indicators are non-interactive and
its text remains understandable without color or animation. Wake is disabled
without valid active wake configuration and when the paired remote is already
connected. Power Toggle sends `KEY_POWER` only through a live paired session.

Remote View keeps **Power Toggle** at the top. With a live paired session, it
sends the toggle and remains in Remote View. Without a live connection to the
selected saved TV, it opens Power View, tries a trusted connection, and starts
Wake if the TV is unreachable and Wake is configured. The disconnected button
is disabled when no active Wake MAC is configured. Remote View then places:

1. The **Directional Pad** as a cross: **Up** above; **Left**
   and **Right** on either side; **Down** below; **Enter** in the center.
2. **Back** and **Home** side by side below the Directional Pad.
3. The volume controls below Back and Home. Put the **Volume Slider**
   above a row containing **Mute**, **Volume Down**, and **Volume Up**.

Each visible control sends the same semantic action as its keyboard equivalent
when one is defined. Keyboard shortcuts must respect text-entry and modal
focus; this document does not assign specific key bindings. A control's enabled
state is derived from the Selected TV, Pairing, Connection, and operation
support together. Disabled controls provide an accessible reason; an icon or
color alone must not carry the reason.

## Startup, states, and feedback

- **First launch:** show the main app window in the Remote View. Do not
  automatically open the Settings Window, display onboarding, or insert a
  setup prompt. Use the standard Remote View layout with disabled remote
  controls. Each disabled control gives an accessible reason such as “No TV
  selected. Open Settings to choose a TV.” The Settings Window opens when the
  user clicks the Settings button.
- **No TVs listed:** hide the TV Selection Table and show **Discover TVs**
  centered both horizontally and vertically in the Settings Main Pane.
  Clicking Discover TVs searches the local network.
- **One TV in the table:** show the TV Selection Table above Discover TVs and
  check the single row's radio button automatically. If it is a Saved TV, make
  it the app's Selected TV. If it is a Discovered TV, begin TV Identity
  Confirmation and pairing; save and use it after both succeed.
- **Multiple TVs in the table:** show Saved TVs and Discovered TVs in one TV
  Selection Table above Discover TVs, with a radio button on each row. Keep
  the active Saved TV checked; if none is active, require the user to select a
  row.
- **Saved TV selected:** make it the app's Selected TV and attempt its
  Connection using the normal reconnect policy.
- **Discovered TV selected:** show TV Identity Confirmation and complete
  pairing before saving it and making it the Selected TV. Do not send remote
  commands to an untrusted, unpaired candidate.
- **Discovery finds no TV or is unavailable:** explain the result, offer a
  retry, and show **Enter TV Address** as the manual fallback. Keep the table
  hidden when there are no rows.
- **Saved TV at launch:** automatically select the previously selected Saved
  TV and attempt its Connection using the normal reconnect policy. If more
  than one TV is saved, resume the most recently selected one. If the TV is
  unavailable, keep it selected and show that a connection attempt is in
  progress or that the Connection failed.
- **TV selected but disconnected or pairing is needed:** preserve the Remote
  View hierarchy and show the relevant state and next action. Disable controls
  that require a live Connection. Offer a Connection retry or a route to TV
  settings as appropriate. Do not conflate Pairing with Connection.
- **Request rejected before admission:** state the specific reason, including
  a busy command queue; do not show a pending indicator for an unaccepted request.
- **Request admitted:** show a brief pending indicator tied to that request.
  Remove it on a terminal result, TV switch, or state resynchronization.
- **Request written:** say “Sent” or “Requested” for the affected operation.
  Do not assert a changed TV state without a matching observation.
- **Request not sent or cancelled before write:** say that the request was not
  sent and explain the next action when one is available.
- **Request outcome uncertain:** state that the TV may or may not have acted;
  keep the affected operation's feedback visible until acknowledged. Do not
  retry it automatically.
- **Observed TV state:** display power, source, mute, or numeric volume only if
  reported by the TV. Mark an old reading as unavailable or unknown after it
  becomes stale; never update a reading optimistically from a button press.
- **Unsupported operation:** make the affected control unavailable and explain
  the limitation when known. Unknown support must not be labeled unsupported.
- **Volume Slider unavailable:** keep it visible but disabled when exact-level
  volume control is unsupported or has not been established. Explain whether
  support is unknown or the TV is known not to support it. Keep Volume Down,
  Volume Up, and Mute usable when those actions are supported. Enable the
  slider only after exact-level writing and a fresh numeric volume reading are
  verified for the Selected TV. Preview a drag locally, then submit one
  exact-level request on release; do not flood the control queue with every
  pointer movement. A slider position must not pretend to be a measured TV
  volume until the TV reports the new value.

TV settings shows a scan-in-progress state with Cancel, then a distinct empty,
permission-denied, timed-out, or failed result. Keep Saved TVs visible during a
failed scan. A discovered row remains a candidate during TV Identity
Confirmation and Pairing; show these steps separately, allow cancellation,
and do not mark it Saved or Selected until both finish successfully. A stale
scan result must not overwrite a newer TV selection. The Settings button stays
available when remote controls are disabled.

Power Toggle sends a toggle only through a live paired remote channel. When
disconnected, it checks the trusted connection briefly and may start Wake;
neither path asserts that the TV is now on or off. Wake's local packet-send
result does not assert that the TV woke. Use each action outcome and observed
TV state separately.

## Global messages

The **Global Messages Pane** is always visible below the current primary view.
It starts about eight text lines high. Render it in a small, readable font and
allow the user to resize its height with the **Messages split bar**. Keep a
minimum usable height for the Remote View; if the window is short, scroll the
Remote View rather than shrink or overlap controls. The split bar is pointer
draggable and has an accessible name and current size.

The Global Messages Pane shows user-relevant messages from all app windows,
including discovery, Pairing, and Connection results from the Settings Window.
Append messages chronologically, newest at the bottom, and provide a vertical
scroll bar when they overflow. When the user is at the bottom, follow new
messages; when the user scrolls upward, preserve their position and show a
new-message indicator instead of forcing a jump. Each row has a fixed-width
severity column and a message-content column. Severity uses its label and
semantic colour, while content is concise user-facing text; do not render
sequence numbers, source bookkeeping, or an idle follow-state label. Keep the
message feed in memory for the current app session; cap its size and remove
oldest entries first. Never include credentials, entered text, network
addresses, or raw payloads. A Settings Window operation can also show a short
local status, but the user-relevant message goes to this shared pane.

## Visual decisions

Use an always-dark graphite interface, blue interaction accents, and semantic
green, amber, and red status styling with text equivalents. The main window
minimum size is 1100 x 760; the Settings Window minimum is 1000 x 660.

P1-M14 makes the Main Window Sidebar a compact icon rail inspired by the
owner-supplied VS Code activity-bar reference. Navigation button slots fill
the rail width without an outer left/right margin or vertical gaps. Their
resting background matches the Sidebar; pointer hover, selection, focus, and
disabled states are visually distinct. Power stays above Remote and Settings
stays at the bottom. Use original local bitmap icons for every Sidebar item,
not copied VS Code artwork. Tooltips and accessible names retain the view
names even when button faces are icon-only.

Each Main Pane primary view uses one bounded-width, horizontally centered
box titled with that view's name. Remote View groups power, directional,
navigation, and volume controls inside the box; Power View groups Wake Steps
and power controls inside its box. Existing placeholder views use the same
titled wrapper. Main Pane action buttons use original local bitmap artwork
where feasible and show pointer-hover feedback. Keep visible text beside
ambiguous Wake/retry/cancel actions, and do not treat the Volume Slider as a
button. The view box scrolls within the primary-view region and does not
include or overlap Global Messages.

TV Settings uses selected-TV, pairing/connection, Wake-setup, and guidance
cards. Setup and recovery actions keep their text even if paired with an icon.
Preserve clear focus indication, keyboard access, and accessible names for
every control; bitmap and hover treatments never replace semantic text or
disabled guidance.
