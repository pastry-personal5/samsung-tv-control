# P1-M14: Remote and Settings GUI Refinement

Status: Planned

## Architecture

P1-M14 is a presentation-only hierarchy refinement. It consumes the existing
typed control state, setup facts, messages, and P1-M13 wake snapshot;
it does not add socket, persistence, protocol, or coordinator policy to Iced.

Create reusable presentation card/style helpers rather than duplicating local
container styling. Power View composes Wake Steps and power actions. Remote
View composes power toggle, directional pad, navigation, then volume. TV Settings
composes selected-TV status, discovery/probe/pairing,
connection recovery, Wake configuration, and saved-TV management cards. The
window scrolls its primary content when necessary; controls never overlap the
Global Messages Pane.

Use the P1-M12 graphite palette consistently. Semantic styling complements,
but does not replace, card headings, state words, tooltip guidance, and
keyboard-operable controls. Maintain stable labels used by existing UI tests
unless a renamed label is documented and tested.

## Tests

- View tests cover card ordering, status/disabled guidance, Wake/Power
  distinction, Settings setup/recovery visibility, and compact-width layout
  behavior at minimum dimensions.
- Keyboard and pointer intents continue to emit the same typed messages.
- Run the contributor-guide Cargo formatting, Clippy, and test gates.
