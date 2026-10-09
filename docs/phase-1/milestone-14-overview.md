# P1-M14: Remote and Settings GUI Refinement

Status: Planned

## Goal

Turn the larger dark interface into a polished control surface: Remote View
uses clear high-contrast control cards, while TV Settings uses scannable setup
cards that make pairing, connection, and Wake configuration easy to understand.

## Scope

- Group Remote View into power toggle, directional, navigation, and volume
  cards with consistent spacing, labels, and disabled explanations;
  do not restore P1-M12's retired lifecycle summary.
- Refine Power View's Wake Steps and power controls, preserving the semantic
  status copy and automatic navigation established in P1-M13.
- Group TV Settings into selected-TV, pairing/connection, Wake setup, and
  concise operational-guidance cards. Retain discovery, probe, pairing,
  re-pair, retry, forget, and saved-TV selection behavior.
- Align Global Messages Pane rows with the dark system's spacing, hierarchy,
  empty state, and semantic severity presentation.

## Out of scope

- New remote capabilities, a dashboard that exposes all details at once,
  hidden disclosure-only setup, source/app/text implementation, or new
  shortcuts.

## Completion checklist

- [ ] Remote controls are grouped into the approved cards and remain usable at
  the P1-M12 minimum window size.
- [ ] Settings cards expose all existing setup/recovery and P1-M13 Wake flows
  without hiding necessary action guidance.
- [ ] Cards and messages retain readable text equivalents for all
  status colors and preserve keyboard access.
- [ ] Presentation tests and documented Cargo gates pass.

## References

- [Architecture](milestone-14-architecture.md)
- [Planned GUI](../ux-gui.md)
