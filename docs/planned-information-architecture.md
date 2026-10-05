# Planned information architecture

Status: Active

This document describes how the first usable app's information is grouped.
It establishes the default view and known relationships, with Wake placement
still open. See the canonical [UX terms](ux-term.md).

## Top-level structure

```text
App window
├── Sidebar
│   ├── Sources → Sources View
│   ├── Apps → Apps View
│   ├── Text Input → Text Input View
│   └── Main Toolbar — at bottom
│       └── Settings button → Settings Window
└── Main Pane
    ├── Remote View — default view
    │   ├── Selected TV context, Pairing and Connection status
    │   ├── Current request feedback and recovery action
    │   ├── Power Toggle
    │   ├── Directional Pad
    │   ├── Back and Home
    │   └── Volume controls
    ├── Global Messages Pane — all user-relevant app messages
    └── Activity View — recent request and Connection events

Settings Window
├── Settings Sidebar
│   └── TV — first item
└── Settings Main Pane
    └── TV settings
        ├── TV Selection Table — when it has one or more rows
        └── Discover TVs button
```

The main app has two main parts: the Sidebar on the left and the Main Pane on
the right. It opens to the Remote View. Selecting Sources, Apps, or Text Input
in the Sidebar displays the corresponding view in the Main Pane. The Global
Messages Pane and Activity View remain below that view. The Main Toolbar sits
at the bottom of the Sidebar and contains a Settings button. The Settings
button opens a separate Settings Window, which has
its own Settings Sidebar and Settings Main Pane. TV is the first Settings
Sidebar item. The lower Main Pane always shows a Global Messages Pane followed
by a separate Activity View. The Global Messages Pane also receives messages
originating in the Settings Window.

## Product areas

| Product area | User purpose | Placement |
| --- | --- | --- |
| Remote View | Send navigation, power, and volume requests to the Selected TV. | Main Pane by default. |
| Sources View | Request a supported source or navigate the TV's source chooser. | Select Sources in the Sidebar. |
| Apps View | Browse and launch apps reported by the Selected TV. | Select Apps in the Sidebar. |
| Text Input View | Send text to a focused native TV text field when supported. | Select Text Input in the Sidebar. |
| Global Messages Pane | Read user-relevant messages from the whole app, including Settings Window results. | Lower Main Pane, above the Activity View. |
| Activity View | Inspect recent request outcomes and Connection events. | Below the Global Messages Pane; always visible. |
| TV setup | Discover a TV or enter its host, pair, and save it for later use. | TV settings in the Settings Window. Open it with the Settings button. |
| Wake | Send a wake request and report whether reconnection succeeds. | TBD. |

Sources, Apps, and Text Input are named Sidebar destinations. Their views
follow the same Selected TV and application monitoring state as the Remote
View; changing views does not change the Selected TV or open another TV
session.

## Context and status

The app opens the main app window to the Remote View, including on first
launch. Do not automatically open the Settings Window or add a separate
onboarding screen. When no TV is selected, keep the main app window visible
with the standard Remote View layout, disable its remote controls, and show a
short status line pointing to Settings. The status line is not a setup prompt.
The Settings Window opens only when the user clicks Settings in the Main Toolbar.

The Remote View is scoped to one Selected TV at a time. Show the selected TV's
identity, Pairing and Connection status, and the most relevant request outcome
near the controls. Keep request outcome separate from any observed TV state.
Show a recovery action in context when the user can act, such as retrying a
Connection or opening TV settings to resolve Pairing. TV selection happens in
TV settings; placement of any additional device management remains TBD.

The Global Messages Pane is a chronological, session-only feed of messages a
user needs to know across both windows. Keep its latest entry at the bottom.
The Activity View shows structured recent command outcomes and Connection
events, also for the current session. A message can summarize an activity
event; both must agree on its outcome. Neither pane stores TV credentials,
entered text, addresses, or raw network data.

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

- Where Wake is accessed.
- Which additional settings pages appear after TV in the Settings Sidebar.
- Where device management and connection details live.

These decisions do not change the agreed two-part window or the Remote View
control hierarchy. Use the exact terms in the [UX glossary](ux-term.md).
