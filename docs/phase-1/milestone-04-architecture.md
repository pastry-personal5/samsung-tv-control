# P1-M4: Iced Application Shell and Navigation

Status: Planned

## Approach

Implement the presentation boundary first, using the accepted UI and state
contracts in [software architecture](../architecture.md) and [planned GUI](../ux-gui.md).
The shell must run without a TV, network permission, preferences, or Keychain
access. It demonstrates navigation and window ownership; it does not imitate
successful TV behavior.

Iced 0.14 supports multiple windows under one application state. Use one Iced
application to keep navigation, window IDs, and the session-only Global
Messages feed shared. The Iced multi-window example demonstrates opening
windows with `window::open`, tracking their IDs, and receiving close events:
[Iced 0.14 multi-window example](https://github.com/iced-rs/iced/blob/master/examples/multi_window/src/main.rs).
Keep long work in `Task` and continuing input or application event observation
in `Subscription`; neither window view owns services. See the current
[Iced application documentation](https://docs.rs/iced/0.14.0/iced/).

## Module responsibilities

- `main.rs`: construct the shell state and run the Iced application. Do not
  put route or widget logic in the composition root.
- `presentation::iced::app`: own Iced window IDs and dispatch Iced messages to
  view-model transitions. Open Settings only when no Settings Window is
  registered; repeated requests while it is open are no-ops. Remove its ID on
  close and leave the main window alive.
- `presentation::iced::message`: define presentation-only events such as
  Sidebar navigation, opening Settings, window opened/closed, splitter resize,
  and message-feed scroll position. Keep these separate from product
  application commands and Samsung protocol messages.
- `presentation::iced::view_model`: own the current Main Pane route, view
  projections, Settings Window visibility, and presentation message state. It
  does not claim a Selected TV; until application services are introduced,
  the projection explicitly represents no Selected TV.
- `presentation::iced::view`: compose Sidebar, Main Toolbar, the current
  primary view, Global Messages Pane, Activity View, and Settings Window page.
  Keep per-view content in small functions or child modules without giving them
  transport access.

The application shell routes `Sources` to `SourcesView`, `Apps` to `AppsView`,
and `Text Input` to `TextInputView`. Route changes update presentation state
only. They do not select a device, start discovery, or open a TV Connection.
Remote View and all three secondary views use the same lower panes. In this
milestone those panes start empty; add safe typed inputs to their presentation
reducers so ordering and scroll behavior can be tested without manufacturing
TV outcomes or network events.

Render the TV settings page with its planned empty layout, but disable Discover
TVs and state that discovery is not available in this shell. Do not invent
discovered rows, open a socket, or request local-network permission. This is a
milestone-only placeholder; the planned enabled discovery flow remains defined
in [the GUI plan](../ux-gui.md).

## State and lifecycle

Use a finite `PrimaryView` enum (`Remote`, `Sources`, `Apps`, `TextInput`) for
route state. The UI derives selected Sidebar styling and content from that
single value. Represent absent device context directly, so disabled controls
cannot accidentally dispatch a placeholder action.

Track the main and Settings `window::Id` values in the Iced application state.
Opening Settings is idempotent. A close event removes the Settings ID; it does
not clear the selected primary view, message feed, or Activity View. The Main
Toolbar remains available from every primary view. Do not create a second Iced
application or an independent Settings process.

The Global Messages Pane uses a bounded, session-only presentation feed. Assign
sequence numbers when accepting typed display-safe messages, append in that
order, and evict the oldest entries at capacity. Keep a scroll position flag:
follow new entries at the bottom, otherwise preserve scroll and surface a
new-message count. Initialize its split height to approximately eight lines;
clamp user resizing so the primary view and Activity View retain usable space.
No persistent height preference is introduced in this milestone.

Activity View starts with a safe empty state. Define its projection boundary
now, but do not add sample entries that imply a TV request occurred. Later
application monitoring can provide typed activity records without changing
the GUI contract.

## Sequencing

1. Add Iced 0.14 and a minimal application entry point; verify that the window
   launches on macOS.
2. Implement one-window route state and the Sidebar destinations. Add the
   Settings Window using Iced multi-window messages and ID tracking.
3. Compose the shared lower panes, split bar, accessible focus order, and
   empty/unselected states.
4. Add deterministic presentation tests for routes, idempotent Settings
   opening, Settings close behavior, bounded message ordering, and scroll
   follow behavior.
5. Run the Cargo gates and visually inspect the main and Settings windows at
   normal and constrained sizes.

Do not introduce domain or infrastructure modules solely to satisfy this
shell milestone. Future application use cases will be injected at the
composition root and projected through the existing presentation boundary.
