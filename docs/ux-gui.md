# Planned GUI

Status: Active

This document describes the first planned GUI for the macOS app. It is a
behavior and hierarchy specification, not a pixel mockup. See the canonical
[UX terms](ux-term.md) and [planned information architecture](planned-information-architecture.md).

## Window structure

The main app window has two main parts:

- **Sidebar** on the left. The **Main Toolbar** is anchored at its bottom and
  contains the **Settings button**; remaining Sidebar contents are TBD.
- **Main Pane** on the right. It opens to the **Remote View**.

The **Settings button** opens a separate **Settings Window**. That window has
its own **Settings Sidebar** and **Settings Main Pane**. The first Settings
Sidebar item is **TV**. Selecting it displays TV settings in the Settings Main
Pane. The Settings Window's other sidebar items and its window modality are
TBD.

```text
┌─────────────────────┬─────────────────────────────────────┐
│ Sidebar             │ Main Pane: Remote View              │
│                     │                                     │
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
├─────────────────────┤                                     │
│ Main Toolbar        │                                     │
│ [Settings]          │                                     │
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

The sketch communicates order and grouping only. It does not set control
dimensions, spacing, colors, icons, or final window size.

## Remote View hierarchy

1. Place **Power Toggle** at the top of the Main Pane.
2. Below it, place the **Directional Pad** as a cross: **Up** above; **Left**
   and **Right** on either side; **Down** below; **Enter** in the center.
3. Place **Back** and **Home** side by side below the Directional Pad.
4. Place the volume controls below Back and Home. Put the **Volume Slider**
   above a row containing **Mute**, **Volume Down**, and **Volume Up**.

Each visible control sends the same semantic action as its keyboard equivalent
when one is defined. Keyboard shortcuts must respect text-entry and modal
focus; this document does not assign specific key bindings.

## Startup, states, and feedback

- **First launch:** show the main app window in the Remote View. Do not
  automatically open the Settings Window, display onboarding, or insert a
  setup prompt. Use the standard Remote View layout; control availability when
  no TV is selected remains TBD. The Settings Window opens when the user clicks
  Settings in the Main Toolbar.
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
  View hierarchy and show the relevant state and next action. Do not conflate
  Pairing with Connection.
- **Request pending:** give visible feedback that a request is being sent.
- **Request outcome unknown:** state that the TV may or may not have acted;
  never claim success based only on a socket write.
- **Unsupported operation:** make the affected control unavailable and explain
  the limitation when known. Unknown support must not be labeled unsupported.
- **Volume Slider unavailable:** when exact-level volume control is not
  supported, show a clear unavailable state for the slider. Keep Volume Down,
  Volume Up, and Mute usable when those actions are supported.

Power Toggle requests a toggle; it does not assert that the TV is now on or
off. Use the action outcome and observed TV state separately.

## Deferred visual decisions

Typography, colors, control dimensions, iconography, window sizing, responsive
breakpoints, animation, and final accessibility copy remain for visual design
and implementation. Preserve clear focus indication, keyboard access, and
accessible names for every control when those details are designed.
