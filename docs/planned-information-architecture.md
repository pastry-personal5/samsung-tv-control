# Planned information architecture

Status: Active

This document describes how the first usable app's information is grouped.
It establishes the default view and known relationships without assigning
unresolved content to the Sidebar. See the canonical [UX terms](ux-term.md).

## Top-level structure

```text
App window
├── Sidebar
│   ├── Main Toolbar — at bottom
│   │   └── Settings button → Settings Window
│   └── Other contents — TBD
└── Main Pane
    └── Remote View — default view
        ├── Selected TV context and operation status
        ├── Power Toggle
        ├── Directional Pad
        ├── Back and Home
        └── Volume controls

Settings Window
├── Settings Sidebar
│   └── TV — first item
└── Settings Main Pane
    └── TV settings
        ├── TV Selection Table — when it has one or more rows
        └── Discover TVs button
```

The main app has two main parts: the Sidebar on the left and the Main Pane on
the right. The app opens to the Remote View. The Main Toolbar sits at the
bottom of the Sidebar and contains a Settings button. Other Sidebar content
remains TBD. The Settings button opens a separate Settings Window, which has
its own Settings Sidebar and Settings Main Pane. TV is the first Settings
Sidebar item.

## Product areas

| Product area | User purpose | Placement |
| --- | --- | --- |
| Remote View | Send navigation, power, and volume requests to the Selected TV. | Main Pane by default. |
| TV setup | Discover a TV or enter its host, pair, and save it for later use. | TV settings in the Settings Window. Open it with the Settings button. |
| Sources | Request a supported source or navigate the TV's source chooser. | TBD. |
| Installed TV apps | Browse and launch apps reported by the Selected TV. | TBD. |
| Text entry | Send text to a focused native TV text field when supported. | TBD. |
| Wake | Send a wake request and report whether reconnection succeeds. | TBD. |

These areas describe product capabilities, not a commitment to a particular
tab, sidebar item, window, or modal. Decide their navigation placement in a
later UX update before implementing those flows.

## Context and status

The app opens the main app window to the Remote View, including on first
launch. Do not automatically open the Settings Window or add a separate
onboarding screen. When no TV is selected, keep the main app window visible;
use the standard Remote View layout without adding a setup prompt. Control
availability in that state remains TBD. The Settings Window opens only when
the user clicks Settings in the Main Toolbar.

The Remote View is scoped to one Selected TV at a time. Show the selected TV's
identity and relevant pairing, connection, and operation status near the
controls, while keeping those states distinct. TV selection happens in TV
settings; placement of any additional device management remains TBD.

When the TV Selection Table has no rows, hide it and center the
**Discover TVs** button horizontally and vertically within the Settings Main
Pane. After a scan, show the Saved TVs and Discovered TVs together in the TV
Selection Table above the button. Each row has a radio button and identifies
whether the TV is Saved or Discovered. Do not show a table when there are zero
TVs.

If the table has one row, check its radio button automatically. If it is a
Saved TV, make it the Selected TV. If it is a Discovered TV, begin TV Identity
Confirmation and pairing; save it and make it the Selected TV after both
succeed. If the table has multiple rows, keep the current Selected TV checked;
if none is active, require the user to choose one. Selecting a Saved TV makes
it the Selected TV. Selecting a Discovered TV starts TV Identity Confirmation
and pairing; after both succeed, save it and make it the Selected TV.
Discovery alone does not establish identity or trust. If discovery is
unavailable or finds no TV, offer **Enter TV Address** in TV settings as the
manual fallback.

When a Saved TV exists, the app selects it automatically at launch. If multiple
TVs have been saved, resume the most recently selected TV, check its radio
button in the TV Selection Table, and attempt its Connection using the normal
reconnect policy. If the TV is unavailable, keep it selected and show the
Connection state. When a feature is unavailable or support is unknown,
communicate that status in the context of the affected operation; do not imply
that every control is usable on every TV.

## Open placement decisions

- Sidebar contents beyond the Main Toolbar and Settings button.
- Where Sources, Installed TV apps, Text entry, and Wake are accessed.
- Which additional settings pages appear after TV in the Settings Sidebar.
- Where device management and connection details live.

These decisions do not change the agreed two-part window or the Remote View
control hierarchy. Use the exact terms in the [UX glossary](ux-term.md).
