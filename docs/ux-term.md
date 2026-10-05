# UX terms

Status: Active

This is the canonical vocabulary for Samsung TV Remote. Product documentation,
interface labels, accessibility descriptions, and implementation-facing UI
names should use these terms consistently. If a new term is needed, define it
here before using it elsewhere.

## Window and navigation

| Term | Meaning and usage |
| --- | --- |
| **Sidebar** | The left-hand part of the main app window. It contains the Main Toolbar at the bottom; other contents remain TBD. |
| **Main Pane** | The right-hand part of the app window. It displays the current view. |
| **Remote View** | The default Main Pane view containing the remote controls. |
| **Main Toolbar** | The toolbar anchored at the bottom of the main app's Sidebar. It contains the Settings button. |
| **Settings button** | Opens the separate Settings Window. |
| **Settings Window** | The separate app window for preferences and TV selection. It has its own Settings Sidebar and Settings Main Pane. |
| **Settings Sidebar** | The navigation sidebar inside the Settings Window. Its first item is TV. Other items are TBD. |
| **Settings Main Pane** | The content area to the right of the Settings Sidebar. It shows the selected settings page. |
| **TV settings** | The Settings Window page opened by selecting the TV item in the Settings Sidebar. |
| **TV Selection Table** | The table in TV settings that lists Saved TVs and TVs found by the latest discovery scan, each with a radio button. Hide it when there are no rows. |
| **Discovered TV** | A TV found on the local network that has not yet been saved as a trusted TV. |
| **Discover TVs** | The action that searches the local network for compatible TVs. Use this verb label for the discovery button. |
| **Enter TV Address** | The manual host-entry action offered when discovery is unavailable or finds no TV. |
| **Device** | A saved app record for a TV, including its local identity and connection preferences. |
| **Saved TV** | A TV whose trusted device record has been saved by the app. |
| **Selected TV** | The TV currently targeted by the Remote View. Say “TV” in user-facing text; use “device” for saved app data and internal concepts. |

## Remote controls

| Term | Meaning and usage |
| --- | --- |
| **Power Toggle** | A control that requests a power toggle. Do not describe it as a confirmed power state or as Power Off. |
| **Directional Pad** | The grouped Up, Left, Right, and Down controls arranged around Enter. |
| **Up**, **Left**, **Right**, **Down** | Directional Pad actions. |
| **Enter** | The center Directional Pad action. It confirms or selects the focused TV item; do not label it Select or OK in the GUI. |
| **Back** | The action that requests the TV's Back command. |
| **Home** | The action that requests the TV's Home command. |
| **Volume Slider** | An interactive control for requesting an exact volume level when the TV supports it. |
| **Mute** | Toggles the TV's mute state. |
| **Volume Down**, **Volume Up** | Step controls that request a decrease or increase in TV volume. |

## Device and capability states

| Term | Meaning and usage |
| --- | --- |
| **Pairing** | The TV approval and credential exchange needed to establish trust. Pairing is distinct from an active connection. |
| **TV Identity Confirmation** | The user's confirmation that a discovered TV is the intended TV, based on the identity details shown by the app. It is separate from approving the pairing prompt on the TV. |
| **Connection** | The current network session between the app and a TV. A paired TV may be disconnected. |
| **Unavailable** | An operation or control cannot currently be used. Explain why when the reason is known. |
| **Unsupported** | The selected TV is known not to support an operation. Do not use this when support is merely unknown. |
| **Unknown support** | The app has not established whether the selected TV supports an operation. Avoid claiming it is unsupported. |

## Writing rules

- Use the exact capitalized terms above for named regions and controls.
- Prefer “TV” in user-facing text and “device” when referring to the saved
  record or internal identity.
- Describe commands as requests unless the TV confirms the resulting state.
- Distinguish pairing state, connection state, and operation support; they are
  separate facts.
- Use **Settings Sidebar** and **Settings Main Pane** only for the Settings
  Window. Use **Sidebar** and **Main Pane** for the main app window.

See [planned information architecture](planned-information-architecture.md)
and [planned GUI](ux-gui.md) for the hierarchy and control layout.
