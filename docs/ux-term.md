# UX terms

Status: Active

This is the canonical vocabulary for the current Samsung TV Remote interface.
The [information architecture](ux-information-architecture.md) owns placement;
the [GUI specification](ux-gui.md) owns visible behavior. Proposed changes to
setup and Wake live in the
[research note](research/discovery-pairing-wake-ux.md).

## Windows and navigation

| Term | Meaning |
| --- | --- |
| **Sidebar** | The icon rail at the left of the main window. Its order is Power, Remote, Sources, Apps, Text Input; the Settings button is anchored at the bottom. |
| **Main Pane** | The main window area to the right of the Sidebar. It holds one primary view above the Global Messages Pane. |
| **Power View** | The primary view for Wake progress and power actions. |
| **Remote View** | The default primary view, containing the visual remote image. |
| **Sources View**, **Apps View**, **Text Input View** | Named primary destinations. They are placeholders in the current release. |
| **Settings button** | The icon-only Sidebar control that opens the separate Settings Window. Its tooltip is “Settings”; the shortcut is ⌘,. |
| **Settings Window** | The separate window for selecting and setting up a TV. |
| **Settings Sidebar** | The Settings Window navigation, with Discovery and Wake on LAN pages. |
| **Settings Main Pane** | The content area beside the Settings Sidebar. |
| **Discovery page** | Settings page for saved TVs, discovery, manual entry, checking, pairing, and connection recovery. |
| **Wake on LAN page** | Settings page for a selected TV's wired and Wi-Fi MAC addresses and active Wake interface. |
| **Global Messages Pane** | The resizable, scrollable, session-only feed below the primary view. It receives user-relevant outcomes from both windows, newest at the bottom. |
| **Messages split bar** | The draggable divider above the Global Messages Pane. |

## TVs and setup

| Term | Meaning |
| --- | --- |
| **Device** | The app's saved record for a TV, including its local identity and preferences. Use “TV” in user-facing copy. |
| **Saved TV** | A TV with a saved trusted device record. A saved TV can be disconnected. |
| **Selected TV** | The saved TV currently targeted by the app. Selecting a candidate does not make it the Selected TV. |
| **Candidate** | An unsaved local address supplied by discovery or manual entry. Discovery alone does not establish identity or trust. |
| **Discover TVs** | Search the local network for TV candidates. |
| **Check TV** | Probe the chosen host's secure endpoint and show the observed certificate SHA-256, plus name/model when available. Checking does not pair. |
| **Pair** | Request trust for a checked candidate and wait for approval on the physical TV. A successful pair saves the TV and makes it the Selected TV. |
| **Re-pair** | Renew trust for a checked Saved TV when its existing pairing needs attention. |
| **Retry Connection** | Try the saved trusted remote connection again; it does not create a new pairing. |
| **Connection** | The current network session with a Saved TV. Pairing and Connection are separate states. |

## Controls and outcomes

| Term | Meaning |
| --- | --- |
| **Power Toggle** | A request to toggle power through a live paired remote. From a disconnected Remote View, it opens Power View and first checks for a trusted connection; Wake may follow if configured and unreachable. It does not indicate measured power state. |
| **Wake** | A deliberate attempt to send one magic packet to the Selected TV's configured active MAC and wait for paired remote readiness. Packet delivery and physical panel state are not confirmed by this action. |
| **Wake Steps** | Non-interactive Power View progress for configuration, packet sending, reconnecting, and remote readiness. |
| **Directional Pad** | Up, Left, Right, and Down around Enter. |
| **Enter** | Confirm the focused TV item; use Enter rather than Select or OK for this action. |
| **Back**, **Home** | TV navigation actions. |
| **Play/Pause** | The new visual remote face. It currently sends Samsung KEY_PLAY. Pause behavior is unverified, so do not describe it as a confirmed toggle. |
| **Mute**, **Volume Down**, **Volume Up** | Mute toggle and step volume requests. There is no exact-volume slider. |
| **Request outcome** | Whether a command was rejected, pending, sent, failed, or uncertain. It does not itself describe the TV's resulting state. |
| **Observed TV state** | A power, source, mute, or volume reading actually reported by the TV. Show freshness when relevant. |
| **Unavailable** | An action cannot currently be used; explain the known reason. |
| **Unsupported** | The TV is known not to support an action. Unknown support is not the same as unsupported. |

## Keyboard shortcuts

These owner-approved shortcuts remain active in Remote View even though the
visual remote has no “Keyboard controls” panel. They are ignored when another
control captures the key and on key repeat. The visual remote's image regions
themselves are pointer-only.

| Action | Shortcut |
| --- | --- |
| Up / Down / Left / Right | ↑ / ↓ / ← / → |
| Enter | Return or Enter |
| Back | Esc |
| Home | Home |
| Mute | M |
| Volume Down / Volume Up | Hyphen / +, including Shift+= |
| Power View / Remote View / Sources View / Apps View / Text Input View | ⌘1 / ⌘2 / ⌘3 / ⌘4 / ⌘5 |
| Open Settings Window | ⌘, |

Power Toggle and Play/Pause have no keyboard shortcut in this milestone. New
shortcuts require an owner decision before implementation.

## Copy rules

- Use the exact names above for named regions and actions. Use “TV” for the
  user-facing object and “device” for its stored record.
- Treat a discovered address, a checked endpoint, a paired TV, and a connected
  session as different facts.
- Describe actions as requests until the TV reports a resulting state. “Remote
  ready” means a paired channel is usable, not that the panel is visibly on.
- Keep routine Settings Window opening and closing out of Global Messages. Show
  user-relevant setup, connection, and command outcomes there.
