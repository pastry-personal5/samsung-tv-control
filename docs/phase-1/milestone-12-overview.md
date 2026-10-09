# P1-M12: Dark GUI, Focused Navigation, and Simplified TV Settings

Status: Done

## Goal

Make the Iced application feel like a deliberate native dark remote: simplify
the main window's navigation and status chrome, make Global Messages the one
compact and readable session feed, give both windows enough room for their
current and planned content, and remove the completed per-key verification
workflow from TV Settings.

## Scope

- Use an always-dark graphite theme: blue is the normal interactive accent;
  green, amber, and red retain semantic status meanings and are never the only
  indication of state.
- Set the main window minimum size to **1100 x 760** and the Settings Window
  minimum size to **1000 x 660**. Preserve the existing initial sizes unless
  the final layout requires them to increase.
- Apply a coherent theme to navigation, cards/containers, buttons, disabled
  controls, text, divider, Global Messages Pane, empty states, and both
  windows. Preserve current keyboard navigation and written disabled-control
  explanations.
- Simplify the main Sidebar:
  - Remove the product name, the “Views: ⌘1–4” hint, and the visible “Main
    Toolbar” label.
  - Keep view navigation as icon-and-label controls (or the existing
    accessible labels where icons are not yet introduced); do not repeat the
    keyboard shortcuts in the Sidebar.
  - Replace the textual “Settings (⌘,)” control with an icon-only **Settings
    button**. Its accessible name, tooltip, and keyboard shortcut remain
    “Settings” and ⌘, respectively.
- Remove the verbose Remote View status block: selected-TV identity, Pairing
  label and guidance, Connection label and guidance, and the aggregate
  “Pair this TV before sending remote actions” style text. Do not remove the
  underlying availability policy or the disabled-control tooltip/rejection
  explanations.
- Retire Activity View and its activity projection from the main window. The
  Global Messages Pane becomes the only session-level user-message surface;
  it must retain user-relevant Settings and main-window outcomes.
- Replace the labelled Messages-height slider and its visible ⌘⇧↑/↓ hint with
  a draggable horizontal split bar immediately above the Global Messages Pane.
  Remove the two pane-resize keyboard shortcuts rather than leaving an
  undiscoverable shortcut. Clamp the resulting pane size to an accessible
  minimum and a maximum that leaves the active Main Pane usable.
- Make Global Messages a bounded, vertically scrollable pane. Its rows use a
  smaller, readable message font and a fixed-width severity column so varying
  labels do not shift message text. Use distinct semantic colours and
  non-colour labels for **Info** and **Warning** (and future severities).
  Do not display feed sequence numbers such as “#1”, “#2”, or “#3”. Present
  concise, user-oriented outcome text rather than transport or presentation
  bookkeeping. Remove the “Following new messages.” status text; retain a
  concise new-message indication only when the reader is away from the bottom.
- Remove the TV Settings instruction beginning “After testing each key on the
  physical TV…” and every individual action-verification button, including
  labels such as “Up: Unverified”.
- Treat all ten `RemoteAction::LIVE_ACTIONS` as verified for every saved TV.
  Pairing, selection, and a usable connection still gate ordinary remote
  actions; manual verification no longer does.
- Delete the `verified_actions` application and presentation behavior and its
  persistence. New saves omit it. Existing JSON files containing it must load
  by ignoring the obsolete field, including files whose obsolete list contains
  the historic `Select` value. This explicitly supersedes P1-M11's
  `Select`-record failure rule because the discarded field has no remaining
  product meaning.

## Out of scope

- Wake, Power Toggle behavior, source/app/text features, new shortcuts, and
  Samsung protocol changes.
- A system/light theme option or a user-selectable theme preference.
- Adding new remote capabilities, Sidebar destinations, or message severities.

## Completion checklist

- [x] Main and Settings windows enforce the approved minimum dimensions.
- [x] The app is always dark and retains readable, non-color-only status
  information in normal, disabled, and empty states.
- [x] The Sidebar contains no product title, Main Toolbar label, or view-key
  hint; its Settings control is icon-only, keyboard reachable, named for
  assistive technology, and still opens Settings with click or ⌘,.
- [x] Remote View contains no selected-TV, Pairing, Connection, or aggregate
  availability text block; unavailable controls still explain why they cannot
  be used.
- [x] Activity View and its presentation projection are absent. User-relevant
  outcomes from both windows remain visible in Global Messages.
- [x] A draggable split bar, rather than a labelled slider or resize shortcut,
  resizes Global Messages within documented bounds.
- [x] Global Messages scrolls vertically; rows have a fixed-width, labelled,
  semantic severity column, smaller readable message text, no sequence
  numbers, and no idle “Following new messages.” text.
- [x] TV Settings contains no manual verification instruction or controls.
- [x] A current or old saved record with or without `verified_actions` loads;
  a subsequent save does not write the field.
- [x] Standard remote actions are no longer rejected or disabled because of
  verification state.
- [x] The documented Cargo validation gates pass.

## References

- [Architecture](milestone-12-architecture.md)
- [Planned GUI](../ux-gui.md)
- [Canonical UX terms](../ux-term.md)
