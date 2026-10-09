# P1-M14: Remote and Settings GUI Refinement

Status: Planned

## Goal

Turn the dark interface into a polished control surface. The Main Window gets
a compact, bitmap-led Sidebar and a horizontally centered, titled view box;
Remote and Power controls remain clearly grouped inside that box. TV Settings
gets scannable setup cards for pairing, connection, and Wake configuration.

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
  rendering path supports it. Preserve each button's accessible name and
  tooltip; do not reuse VS Code's artwork.
- Give each Main Pane primary view one bounded-width box, horizontally
  centered in the primary-view region. Put the view title in that box once,
  not as a duplicate heading outside it. Remote View groups power toggle,
  directional, navigation, and volume controls within the box, with consistent
  spacing and disabled explanations; do not restore P1-M12's retired lifecycle
  summary.
- Refine Power View's Wake Steps and power controls inside its titled box,
  preserving the semantic status copy and automatic navigation established in
  P1-M13. Placeholder views use the same titled-box pattern without adding
  capabilities.
- Create an original bitmap for each implemented Main Pane action button and
  use it as the visible button artwork where feasible. Standard remote symbols
  may be icon-only with accessible names/tooltips; ambiguous actions such as
  Wake, Try again, and Cancel retain visible text with their bitmap. Give
  enabled action buttons pointer-hover feedback without hiding focus or
  disabled states. The Volume Slider is not a button and retains its own
  affordance.
- Group TV Settings into selected-TV, pairing/connection, Wake setup, and
  concise operational-guidance cards. Retain discovery, probe, pairing,
  re-pair, retry, forget, and saved-TV selection behavior. Where an icon helps
  a Settings action, pair bitmap artwork with its visible text rather than
  making safety/recovery actions icon-only.
- Align Global Messages Pane rows with the dark system's spacing, hierarchy,
  empty state, and semantic severity presentation.

## Out of scope

- New remote capabilities, a dashboard that exposes all details at once,
  hidden disclosure-only setup, source/app/text implementation, or new
  shortcuts. The Settings Window Sidebar is not converted into the Main Window
  icon rail in this milestone.

## Completion checklist

- [ ] Remote controls are grouped into the approved cards and remain usable at
  the P1-M12 minimum window size.
- [ ] Main Sidebar slots are full width with zero inter-button gap, a
  Sidebar-matched resting background, and visible hover/selected/focus states;
  Power remains above Remote and Settings remains bottom-anchored.
- [ ] Every Main Sidebar destination has original bundled bitmap artwork;
  every implemented Main Pane button has its planned bitmap treatment or a
  documented feasibility/accessibility exception. Text names, tooltips, and
  keyboard navigation continue to work without interpreting the picture.
- [ ] Each Main Pane view has one titled box centered horizontally in the
  primary-view region; nested control groups do not overlap the Messages
  split bar or require horizontal scrolling at the minimum window size.
- [ ] Settings cards expose all existing setup/recovery and P1-M13 Wake flows
  without hiding necessary action guidance.
- [ ] Cards and messages retain readable text equivalents for all
  status colors and preserve keyboard access. Hover feedback is supplementary.
- [ ] Native macOS review covers resting, hovered, selected, focused, and
  disabled buttons at the minimum window size and a Retina display scale;
  presentation tests and documented Cargo gates pass.

## References

- [Architecture](milestone-14-architecture.md)
- [Planned GUI](../ux-gui.md)
