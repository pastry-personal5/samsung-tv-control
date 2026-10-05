# P1-M3: UX Terms, Information Architecture, and GUI

Status: Done

## Approach

This milestone produces product-facing design documentation only. Treat
`docs/ux-term.md` as the canonical source for interface terminology. The
information architecture and GUI documents use those terms and link back to
the glossary rather than introducing synonyms.

Keep the documentation aligned with the existing product and software
architecture: the first usable app supports TV discovery and setup, core remote
controls, wake, sources, installed TV apps, and text input where supported.
This milestone defines the Main Toolbar's Settings button, a separate Settings
Window with TV first in its Settings Sidebar, TV selection, and discovery
within TV settings. First launch opens the main app window directly, without a
setup prompt or automatic Settings Window. With no Selected TV, remote controls
are disabled and a short status line points to Settings. The lower Main Pane
contains an always-visible, resizable Global Messages Pane and a separate,
always-visible Activity View below it. TV settings uses a TV Selection
Table above Discover TVs, hiding the table when empty, preselecting a single
row, and requiring a radio-button choice among multiple unselected TVs.
Selecting a Discovered TV requires identity confirmation and pairing before it
is saved and used. The Sidebar includes Sources, Apps, and Text Input items
that open their corresponding Main Pane views. Wake placement remains TBD.

## Documentation boundaries

- `ux-term.md` defines the preferred user-facing terms, concise meanings, and
  distinctions that matter to users, including power toggle versus confirmed
  power state and TV pairing versus network connection.
- `planned-information-architecture.md` maps product areas and their
  relationships, identifies the default view and named Sidebar destinations,
  Settings Window and TV setup, and records remaining TBD placements and
  no-selected-TV behavior without pretending they are decided.
- `ux-gui.md` describes the two-part window and Remote View hierarchy, control
  order, Settings Window hierarchy, discovery flow, the two lower monitoring
  regions, and empty, unavailable, and capability-dependent states. It does
  not specify Iced widget APIs or pixel dimensions.

No public Rust interfaces or application types change in this milestone. GUI
implementation remains a later milestone.
