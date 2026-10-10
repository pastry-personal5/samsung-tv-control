# P1-M14: Remote and Settings GUI Refinement

Status: Done

## Goal

Turn the dark interface into a polished control surface. The Main Window gets
a compact, bitmap-led Sidebar and a horizontally centered, titled view box.
The owner selected a visual remote image with clickable control regions for
Remote View, without a second control panel below it. Power controls remain
grouped inside their view box. Settings has scannable Discovery and Wake on LAN
pages.

The supplied VS Code activity-bar screenshot is a reference for density,
edge-to-edge hit areas, and interaction states, not an asset to copy.

## Scope

- Make the Main Window Sidebar a compact icon rail. Each navigation button
  fills the rail width, with no outside left/right margin and no vertical gap
  between adjacent button slots. Resting buttons use the Sidebar background;
  pointer hover, selected, keyboard focus, and disabled states remain visibly
  distinct. Keep Settings anchored at the bottom.
- Create original, locally bundled bitmap icons for every Sidebar destination
  (Power, Remote, Sources, Apps, Text Input) and Settings. Replace the visible
  text/glyph face of those buttons with the bitmaps where the Iced/macOS
  rendering path supports it. Keep semantic names and tooltips in code; native
  screen-reader exposure remains unverified. Do not reuse VS Code's artwork.
- Give each Main Pane primary view one bounded-width box, horizontally
  centered in the primary-view region. Put the view title in that box once.
  Remote View shows only the bundled visual remote. The image highlights
  hovered controls and dims unavailable controls. Do not restore P1-M12's
  retired lifecycle summary or add duplicate control buttons below the image.
- Refine Power View with Power Controls above Wake Steps inside its titled box,
  preserving the semantic status copy and automatic navigation established in
  P1-M13. Placeholder views use the same titled-box pattern without adding
  capabilities.
- Use original local bitmap artwork for implemented action buttons where
  available. The visual remote is a single embedded image with tested pointer
  regions. Wake, Try again, and Cancel retain visible text and bitmaps in
  Power View. Enabled buttons and picture regions respond to pointer hover.
- Add the owner's Play/Pause face to the visual remote. It sends Samsung
  `KEY_PLAY` when playback is unknown or inferred paused and `KEY_PAUSE` when
  inferred playing. The remote channel does not report independent playback
  state.
- Group the Discovery page into TV List, Discovery and Pairing, and Guidance
  cards. Put Check TV, Re-pair, Retry Connection, and Forget Selected TV in
  the selected Saved TV row. Keep Wake configuration and its Guidance on the
  separate Wake on LAN page. Retain discovery, probe, pairing, re-pair, retry,
  forget, and saved-TV selection behavior. Where an icon helps a Settings
  action, pair artwork with visible text.
- Align Global Messages Pane rows with the dark system's spacing, hierarchy,
  empty state, and semantic severity presentation.

## Out of scope

- Further remote capabilities beyond the owner's Play/Pause addition, a
  dashboard that exposes all details at once, hidden disclosure-only setup,
  source/app/text implementation, or new shortcuts. The Settings Window
  Sidebar is not converted into the Main Window icon rail in this milestone.

## Completion checklist

- [x] Remote View contains the visual remote without a duplicate keyboard
  control panel; its pointer regions emit typed actions, and the primary view
  scrolls at the P1-M12 minimum window size.
- [x] Main Sidebar slots are full width with zero inter-button gap and a
  Sidebar-matched resting background; Power remains above Remote and Settings
  remains bottom-anchored.
- [x] Native review verifies the resting, hovered, and selected Sidebar states
  at the minimum window size. Iced's macOS accessibility tree exposes the
  window but not individual controls, and keyboard focus is not exposed for
  these icon-only controls; both remain documented platform follow-ups.
- [x] Every Main Sidebar destination has original bundled bitmap artwork.
  Main Pane controls use the bundled image or original button bitmaps.
- [x] Each Main Pane view has one titled box centered horizontally in the
  primary-view region. Remote, Power, Sources, Apps, and Text Input were
  inspected at the minimum size with no Messages overlap.
- [x] Settings cards expose all existing setup/recovery and P1-M13 Wake flows
  without hiding necessary action guidance.
- [x] Power cards and messages retain readable text equivalents for status
  colors. Existing Remote View shortcuts still work for their mapped actions;
  the image's regions are pointer-only, including Play/Pause.
- [x] Presentation tests and documented Cargo gates pass; the signed app
  bundle builds and verifies.
- [x] Native macOS review covers resting, hovered, selected, and disabled
  controls at the minimum window size and Retina scale. Keyboard focus remains
  unavailable in this Iced/macOS build and is recorded as a product follow-up.
- [x] Play/Pause resolves to `KEY_PLAY` from unknown or inferred paused state
  and `KEY_PAUSE` from inferred playing state after a confirmed write.
- [x] The app makes no observed-player-state claim. It records only inferred
  playback state after a confirmed command write; live-TV state observation is
  deferred to a supported monitoring integration.

## Validation evidence

- `make bundle` produced and verified the signed macOS app on 2026-10-10.
- Native review inspected all primary views, Settings Discovery, the selected
  Saved TV action row, Power Controls above Wake Steps, sidebar selection and
  hover feedback, and disabled Power controls at the 1100 × 760 minimum size.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
  and `cargo test` passed after the final changes.

## References

- [Architecture](milestone-14-architecture.md)
- [GUI specification](../ux-gui.md)
