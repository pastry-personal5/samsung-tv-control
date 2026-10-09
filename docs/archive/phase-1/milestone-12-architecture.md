# P1-M12: Dark GUI, Focused Navigation, and Simplified TV Settings

Status: Archived

## Architecture

Keep the existing dependency direction. Theme selection and widget styling
remain presentation responsibilities; application code exposes only the facts
that views need. Configure Iced's application builder with its dark theme and
use shared presentation style helpers for the graphite surfaces and semantic
states. Do not spread literal colors across individual view functions.

The window settings remain owned by `presentation::iced::app`: main minimum
size is 1100 x 760 and Settings minimum size is 1000 x 660. The larger layout
keeps the active Main Pane and Global Messages Pane distinct. Activity View is
retired; it is not hidden behind a feature flag or retained as an empty
placeholder.

## Main-window simplification

`presentation::iced::view` removes the product-title text, Main Toolbar text,
and “Views: ⌘1–4” hint from the Sidebar. The visual Settings control becomes
an icon-only button, while its accessible name and tooltip remain “Settings”; a
visible label is not required for a control with an unambiguous icon and an
accessible text alternative. `Shortcut::OpenSettings` continues to provide
⌘,. Do not retain a text label merely to expose the shortcut.

Remove the Remote View's selected-TV, Pairing, Connection, lifecycle-guidance,
and aggregate availability lines. This is a presentation reduction only:
`ControlState` remains authoritative and `RemoteControlViewState` continues to
derive each control's enabled state and user-facing disabled reason. Tooltips
and rejected-action messages remain the recovery path for a blocked control.

Remove the activity collection, its projection, and its view rendering where
they solely support Activity View. Preserve the bounded app-wide message feed;
both main and Settings window results continue to append typed presentation
messages to it.

## Global Messages Pane

The Main Pane places a pointer-draggable horizontal split bar directly before
the Global Messages Pane. It replaces the labelled `slider` widget and the
increase/decrease message-height shortcuts. The presentation model owns the
clamped pane height while the split-bar event maps pointer position to that
model; it must not expose raw window events to application code.

Render the feed in a vertically scrollable viewport with an always-available
scroll bar when content overflows. A message row has a fixed-width severity
column followed by a wrapping message-content column. The severity cell uses
both its text label (currently **Info** or **Warning**) and semantic colour;
do not rely on colour alone. The content column renders the already-sanitized,
user-oriented message text at a smaller but readable size. Sequence identifiers
and presentation-bookkeeping source strings are not rendered. Keep the bounded
feed, ordered append semantics, auto-follow-at-bottom behavior, and the
new-message indicator while reading earlier entries; remove the idle
“Following new messages.” label.

Add a presentation formatter at the typed message boundary rather than
assembling `#sequence · severity · source · text` strings in the view. It
receives `DisplayMessage` fields and returns the row's display-ready severity
and concise user message. The formatter must not reveal addresses, credentials,
tokens, raw protocol frames, or internal request identifiers.

## Retiring manual verification

`RemoteAction::LIVE_ACTIONS` is the complete standard action set. Remove
`verified_actions` from `SavedDevice`, the repository contract, setup flow,
control-state capability checks, Iced messages, view-model projections, and
the Settings view. Ordinary-action availability depends only on the selected
device, current selection generation, Pairing, and Connection facts.

Remove `verified_actions` from the disk record. Serde's normal unknown-field
handling then ignores it when reading an existing record, so neither `Enter`
nor historical `Select` requires enum decoding. The next atomic repository
write strips the obsolete JSON member. This is an intentional, safe semantic
retirement of M11's action-list persistence rather than a general legacy-data
migration; trust, credentials, device identity, host, and selection behavior
remain unchanged.

## Tests

- Assert both `window::Settings` minimum sizes and the always-dark application
  theme/style projections.
- Assert the Sidebar's Settings button has an accessible Settings name/tooltip
  without a visible text label, and that its shortcut still opens Settings.
- Assert Remote View omits lifecycle summary text while the corresponding
  unavailable controls retain their specific disabled reasons.
- Assert no Activity View projection or rendering remains, while main-window
  and Settings-window outcomes append to the same Global Messages feed.
- Exercise split-bar resize clamping and verify the removed resize shortcuts
  have no effect.
- Render Info and Warning message rows and assert their fixed severity-column
  width, semantic style, smaller message text, absence of sequence/source
  bookkeeping, scrollability, and away-from-bottom new-message indication.
- Assert no view-model state or Settings message mutates action verification.
- Exercise ordinary action admission for every standard action after a usable
  connection without a verification capability.
- Deserialize records that omit `verified_actions`, contain current `Enter`,
  and contain historical `Select`; verify a save writes no such field.
- Run the contributor-guide Cargo formatting, Clippy, and test gates.
