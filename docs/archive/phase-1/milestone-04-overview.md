# P1-M4: Iced Application Shell and Navigation

Status: Archived

## Goal

Deliver a launchable macOS GUI shell that follows the agreed window structure,
routes Sidebar selections to their views, and reserves shared space for
messages and recent activity.

## Sequence

Start after P1-M3 is marked Done. This milestone builds the presentation shell
only; it does not connect to a TV or implement product use cases.

## Scope

In scope:

- Add the selected Iced dependency and launch the app in a native macOS window.
- Show Remote View at startup with no Selected TV; keep controls visible and
  disabled with the planned Settings status.
- Add Sources, Apps, and Text Input Sidebar items that route to Sources View,
  Apps View, and Text Input View in the Main Pane.
- Add the separate Settings Window with the TV settings page shell. Opening it
  repeatedly must not create duplicate Settings Windows; closing it leaves the
  main app window available. Show the Discover TVs entry point as disabled
  until discovery is implemented; do not simulate scan results or trigger a
  network request.
- Keep the Global Messages Pane and Activity View below every primary view.
  Implement the resizable message pane, its initial height, its scroll
  behavior, and safe empty states.
- Keep navigation state and window tracking in the presentation shell. Keep
  the view layer free of network and storage calls.
- Add deterministic tests for route selection, window-open/close state, and
  message-feed ordering and scroll-follow behavior where practical.

Out of scope:

- TV discovery, manual host validation, Pairing, Connection, and protocol work.
- Sending remote actions, changing sources, launching apps, or sending text.
- Persisting devices, messages, window sizes, or navigation state.
- Final visual styling, icons, animation, and distribution/signing.

## Completion checklist

- [x] `cargo run` opens the main Iced window in Remote View.
- [x] No Selected TV state displays disabled controls and a Settings status;
  no screen makes a network request.
- [x] Each of the three Sidebar items opens its matching Main Pane view and
  leaves device context unchanged and preserves the shared lower panes.
- [x] Settings opens as a separate window, does not duplicate on repeated
  activation, and can close without closing the main window. Its discovery
  entry point is visibly unavailable and makes no network request.
- [x] The Global Messages Pane starts at about eight text lines, can be resized
  with the split bar, and follows new messages only when the user is already
  at the bottom. The Activity View remains visible below it.
- [x] The shell provides keyboard shortcuts for the four views, Settings, and
  message-pane resizing. Disabled actions show a visible reason and tooltips.
- [x] Tests cover view routing and window lifecycle policy; empty states are
  verified for each view.
- [x] Run the required Cargo format, Clippy, and test gates from the
  [contribution guide](../../contribution-guide.md).
- [x] Visually inspect the main and Settings windows in a native macOS session,
  including a constrained main window.
- [x] Update the phase status and changelog with acceptance evidence.

## Acceptance evidence (2026-10-08)

The macOS QA build opened in Remote View. Native interaction verified Sources,
Apps, and Text Input empty states; the shared message and activity panes
remained visible. Settings opened with `⌘,` and closed while the main window
remained available. At the 700×560 minimum window size, the primary view
scrolled and the lower panes remained visible. `⌘1`–`⌘4` navigated among
views, and `⌘⇧↑` increased the message pane height. Tests cover route state,
Settings lifecycle, bounded feed ordering, scroll-follow policy, and shortcut
mapping.

## Accessibility follow-up

The owner chose to preserve Iced and revise this shell milestone's acceptance
to the keyboard shortcuts and visible disabled reasons verified above. In the
current Iced 0.14 build, macOS exposes the app window in its accessibility tree
but does not expose the buttons or slider. Native keyboard focus order and
screen-reader control names remain product accessibility work before release;
this milestone does not claim them.

When a change review is opened, attach sanitized main and Settings screenshots
as required by the [contribution guide](../../contribution-guide.md). No change
review was opened for this milestone completion pass.
