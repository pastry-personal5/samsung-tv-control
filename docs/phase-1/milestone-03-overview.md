# P1-M3: UX Terms, Information Architecture, and GUI

Status: Done

## Goal

Define shared UX language, information hierarchy, and the planned first GUI
layout before implementation.

## Scope

In scope:

- Establish canonical UX terms for use throughout product documentation and
  implementation.
- Describe the product's information hierarchy and identify unresolved
  placement decisions.
- Specify the Remote View's two-pane structure and requested control layout.
- Specify the Main Toolbar, Settings Window, TV selection, and discovery flow
  within TV settings.
- Define TV Selection Table visibility and single/multiple-row selection
  behavior for saved and discovered TVs.
- Define key empty, unavailable, and capability-dependent GUI states.
- Define the Global Messages Pane and separate Activity View, including their
  shared message source and order.
- Define Sidebar destinations and their corresponding Main Pane views for
  Sources, Apps, and Text Input.

Out of scope:

- Implementing Iced views, application behavior, or device controls.
- Pixel-level measurements, visual styling, icons, and final copywriting.
- Deciding where Wake appears; that placement remains open.

## Completion checklist

- [x] Publish a canonical glossary in [UX terms](../ux-term.md).
- [x] Publish the product hierarchy and explicit TBD decisions in
  [planned information architecture](../planned-information-architecture.md).
- [x] Publish the Remote View layout and interaction states in
  [planned GUI](../ux-gui.md).
- [x] Specify cross-window messages and recent activity without exposing
  sensitive TV or local-network data.
- [x] Use the glossary terms consistently across all three UX documents.
- [x] Link the documents from this plan and record owner decisions in the
  [phase changelog](changelog.md).
- [x] Check relative Markdown links and `git diff --check`.

## Acceptance evidence

- UX terms, information hierarchy, and GUI behavior are published in the
  linked canonical documents. They include the agreed first-launch state,
  Sidebar routes, TV selection flow, Global Messages Pane, and Activity View.
- Owner decisions are recorded in the [phase changelog](changelog.md).
- `cargo fmt --all -- --check` and
  `cargo clippy --all-targets -- -D warnings` passed.
- `cargo test` passed; the scaffold currently contains zero tests, which is
  expected because P1-M3 adds no application behavior.
- All relative Markdown links resolved, and `git diff --check` passed.
