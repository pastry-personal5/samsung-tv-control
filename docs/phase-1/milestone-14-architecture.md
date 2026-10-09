# P1-M14: Remote and Settings GUI Refinement

Status: Active

## Architecture

P1-M14 primarily refines presentation hierarchy. It consumes the existing
typed control state, setup facts, messages, and P1-M13 wake snapshot; it does
not add socket, persistence, or coordinator policy to Iced.
The owner later selected a bundled visual remote for Remote View and added one
Play/Pause intent. The existing Samsung codec sends `KEY_PLAY` for that intent;
pause behavior awaits a live-TV check.

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

The bundled image path and Retina rendering need native review. Store
original PNG artwork under `assets/icons/`, with a fixed logical size and
at least 1×/2× raster variants or an equivalent high-density source. Keep the
bitmap independent of hover and selected backgrounds so styling can be
consistent. Use stable semantic names, not positional names, for the five
Sidebar destinations, Settings, and each implemented Main Pane action. The
Remote View image is embedded from `assets/remote-concept-v5.png`; its pointer
regions are mapped in `remote_image.rs`. The image supplies the Remote View
artwork; no second row or panel of action buttons is rendered below it.

The supplied VS Code image informs density and state treatment only; draw
distinct icons and do not copy, crop, or ship the reference or branded art.
Prefer embedded or explicitly bundled local bytes so no icon depends on the
working directory or network. Verify a release `.app` contains and displays
every asset.

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
View shows only the centered visual remote. The other primary views use the
same titled wrapper while remaining placeholders. The image's pointer regions
show hover and unavailable overlays. Iced canvas regions are not individual
focus targets. Existing app shortcuts remain available for their mapped
actions; Play/Pause on the image has no keyboard equivalent in this milestone.
Ambiguous or consequential Power View actions retain visible text beside the
bitmap. Power View does not use image color alone to explain unavailable
actions. The Settings Discovery page contains saved-TV selection,
discovery/probe/pairing, connection recovery, and guidance cards. The separate
Wake on LAN page contains Wake configuration and guidance. Setup and recovery
buttons retain visible text even when they gain artwork.

Use the P1-M12 graphite palette consistently. Semantic styling complements,
but does not replace, card headings, state words, tooltip guidance, and
keyboard-operable controls. Enabled native Main Pane buttons get a clear
pointer-hover change to the button surface without changing their command,
while disabled buttons do not appear actionable. Hover, focus, pressed,
selected, and disabled native states must remain distinguishable in the dark
palette.
Maintain stable semantic labels used by existing UI tests even when visible
button faces become bitmaps; document and test any intentional label change.

## Tests

- Presentation tests cover Sidebar order, bottom-anchored Settings, selected
  projection, view-box title/ordering, status/disabled guidance, Wake/Power
  distinction, Settings setup/recovery visibility, and minimum-width layout.
- Interaction tests cover hover/focus/disabled styles for native Sidebar and
  Power View buttons. Picture-region tests cover distinct hotspots, gaps,
  coordinate mapping, and disabled controls. Asset tests cover the icon
  inventory and embedded PNG headers.
- Native signed-bundle acceptance checks Sidebar edge-to-edge hit areas and
  zero inter-item gap; default/hover/selected/focused/disabled states; all
  bitmap artwork at normal and Retina scale; centered boxes; scroll behavior;
  and no overlap with Global Messages at minimum window dimensions.
- Run the contributor-guide Cargo formatting, Clippy, and test gates.

Full native review of Settings, placeholder views, focus, and all Sidebar
states is still needed. A socket write of `KEY_PLAY` does not prove Play/Pause
behavior; that action needs a live-TV check before pause can be claimed.
The signed bundle's Remote and Power Views were inspected at the minimum
window size, including scrolling without Messages overlap. The macOS
accessibility tree exposed only the window, so individual screen-reader labels
and a distinct keyboard-focus ring have not been verified. The image-only
layout leaves those actions pointer-only; this is a known
accessibility limit under the P1-M4 owner-approved follow-up.
