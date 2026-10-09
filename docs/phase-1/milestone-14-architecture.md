# P1-M14: Remote and Settings GUI Refinement

Status: Planned

## Architecture

P1-M14 is a presentation-only hierarchy refinement. It consumes the existing
typed control state, setup facts, messages, and P1-M13 wake snapshot;
it does not add socket, persistence, protocol, or coordinator policy to Iced.

### Main Sidebar

Replace the current padded, spaced text-button column with one narrow rail in
the Main Window only. A shared Sidebar-item helper owns the full-width hit
rectangle, fixed consistent row height, zero column spacing, and no outer
horizontal inset around item slots. Internal padding may center artwork while
keeping a comfortable pointer target. The resting item surface exactly matches
the rail background; hover changes the item's surface, selected state remains
persistent, and keyboard focus has a distinct visible outline. Hover must not
be the only way to identify the selected view. Keep the Power, Remote, Sources,
Apps, Text Input order, a flexible spacer, then Settings at the bottom.

The buttons are visually icon-only, but their names remain in the navigation
model, accessible labels, and tooltips. The Power button selected by the
P1-M13 disconnected-toggle flow must use the same persistent selected style as
a direct Sidebar click. Retain keyboard navigation and existing shortcuts;
do not add new bindings. The separate Settings Window Sidebar is unchanged.

### Bitmap artwork and loading

First verify a single transparent bitmap in the pinned Iced/macOS build,
including its bundled-app loading path and Retina rendering. If the image
widget/feature or bundle pipeline needs configuration, make that change in
presentation/build configuration before producing the full icon set. Store
original PNG artwork under `assets/icons/`, with a fixed logical size and
at least 1×/2× raster variants or an equivalent high-density source. Keep the
bitmap independent of hover and selected backgrounds so styling can be
consistent. Use stable semantic names, not positional names, for the five
Sidebar destinations, Settings, and each implemented Main Pane action
(Power Toggle, Wake, Try again, Cancel, directions, Enter, Back, Home, Mute,
Volume Down, and Volume Up). Inventory the actual rendered controls before
finalizing assets so none silently remain text-only.

The supplied VS Code image informs density and state treatment only; draw
distinct icons and do not copy, crop, or ship the reference or branded art.
Prefer embedded or explicitly bundled local bytes so no icon depends on the
working directory or network. Verify a release `.app` contains and displays
every asset. If bitmap rendering is not viable after the spike, document the
specific blocker and retain named, keyboard-accessible buttons with a
code-native icon treatment; do not ship missing-image placeholders or silently
drop a button.

### Centered primary-view box and cards

Create reusable view-box, card, icon-button, and interaction-style helpers
instead of duplicating container styling. Wrap each primary view in one
bounded-width box centered horizontally inside the existing scrollable
primary-view area. Its heading is the view title (Power View, Remote View,
etc.) and appears once inside the box. This does not vertically center the
content or include the Messages split bar and Global Messages Pane. At the
1100 × 760 minimum, the box fits the width without horizontal scrolling;
smaller available heights scroll the primary-view area rather than compress
controls into the message region.

Power View composes Wake Steps and power actions inside its outer box. Remote
View composes power toggle, directional pad, navigation, then volume as
internal cards. The other primary views use the same titled wrapper while
remaining placeholders. Standard remote controls may show bitmap-only faces
because their name is present in accessibility and hover/focus tooltips.
Ambiguous or consequential Power View actions retain visible text beside the
bitmap. Do not use image color alone to explain unavailable actions. TV
Settings composes selected-TV status, discovery/probe/pairing, connection
recovery, Wake configuration, and saved-TV management cards; setup and
recovery buttons retain visible text even when they gain artwork.

Use the P1-M12 graphite palette consistently. Semantic styling complements,
but does not replace, card headings, state words, tooltip guidance, and
keyboard-operable controls. Enabled Main Pane action buttons get a clear
pointer-hover change to the button surface without changing their command,
while disabled buttons do not appear actionable. Hover, focus, pressed,
selected, and disabled states must remain distinguishable in the dark palette.
Maintain stable semantic labels used by existing UI tests even when visible
button faces become bitmaps; document and test any intentional label change.

## Tests

- Presentation tests cover Sidebar order, bottom-anchored Settings, selected
  projection, view-box title/ordering, status/disabled guidance, Wake/Power
  distinction, Settings setup/recovery visibility, and minimum-width layout.
- Interaction tests cover hover/focus/disabled styles and that icon buttons
  still emit the same typed messages by pointer or keyboard. Asset tests cover
  the icon inventory, decoding, and absence of missing-image fallback.
- Native signed-bundle acceptance checks Sidebar edge-to-edge hit areas and
  zero inter-item gap; default/hover/selected/focused/disabled states; all
  bitmap artwork at normal and Retina scale; centered boxes; scroll behavior;
  and no overlap with Global Messages at minimum window dimensions.
- Run the contributor-guide Cargo formatting, Clippy, and test gates.
