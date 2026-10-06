# Phase 1 Changelog

Status: Active

## Entries

- 2026-10-07 — Reviewed and corrected P1-M5 contracts: renamed the request
  `SendRemoteAction`, retained `DeviceId` inside the safe display projection,
  and made `PowerToggle` explicit. Removed the duplicate integration-test
  harness. `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D
  warnings`, and `cargo test` pass (22 tests).
- 2026-10-07 — Planned P1-M7 through P1-M9 after P1-M6: in-memory device
  selection with generation tracking, distinct pairing/connection state
  projection, then typed remote-command admission policy. Each milestone is
  limited to deterministic application and presentation behavior; transport,
  persistence, discovery, and TV protocol work remain for later planning.
- 2026-10-06 — Planned P1-M5 and P1-M6 as deliberately small foundation
  milestones after the completed presentation shell. P1-M5 introduces only
  typed domain/application control contracts. P1-M6 makes the existing
  no-selected-TV state an application-owned control gate and projects it into
  Iced. Neither milestone begins discovery, persistence, pairing, a network
  connection, or Samsung protocol work. Phase 1 is Active again until these
  planned milestones are completed.
- 2026-10-06 — P1-M4 implementation added the Iced presentation shell with
  in-memory routing, idempotent multi-window Settings lifecycle, disabled
  no-TV controls, resizable shared messages/activity regions, and deterministic
  presentation-state tests. Cargo validation passed; native-window visual
  acceptance and sanitized screenshots remain pending on a visible macOS
  session. No network or storage service is present in the shell.
- 2026-10-05 — Completed P1-M3 after publishing the canonical UX glossary,
  information architecture, GUI interaction plan, and control/monitoring
  design. Verified Cargo format, Clippy, and test gates; `cargo test` passed
  with zero tests because this milestone adds no behavior. Relative Markdown
  links and `git diff --check` passed. See the completed
  [milestone overview](milestone-03-overview.md).
- 2026-10-05 — Planned P1-M4, Iced Application Shell and Navigation, as the
  next implementation milestone after P1-M3 completes. It builds the shared
  window and view shell without TV networking or device behavior. The plan
  uses one Iced application for the main and Settings windows and keeps
  presentation state separate from application commands. See the
  [overview](milestone-04-overview.md) and
  [architecture](milestone-04-architecture.md).
- 2026-10-05 — Owner added **Sources**, **Apps**, and **Text Input** as Sidebar
  destinations. Each opens Sources View, Apps View, or Text Input View in the
  Main Pane while the Global Messages Pane and Activity View remain available.
  Wake placement remains open. Updated the [UX glossary](../ux-term.md),
  [information architecture](../planned-information-architecture.md),
  [GUI plan](../ux-gui.md), and P1-M3 plans.
- 2026-10-05 — Reviewed control, monitoring, and Clean Architecture boundaries.
  Assigned the sole bounded command queue and result journal to the application
  coordinator, kept socket writes in the Samsung adapter, and defined atomic
  snapshots, sequenced events, stale observations, and request outcomes. The
  owner chose disabled remote controls with a short Settings status on first
  launch, a visible but disabled Volume Slider when exact control is unavailable,
  and two always-visible lower Main Pane regions: a resizable Global Messages
  Pane (about eight lines initially, newest message at bottom) above a separate
  Activity View. Messages from both app windows share a session-only feed.
  Updated the [software architecture](../architecture.md),
  [repository map](../planned-repository-architecture.md),
  [UX glossary](../ux-term.md),
  [information architecture](../planned-information-architecture.md), and
  [GUI plan](../ux-gui.md).
- 2026-10-05 — Owner clarified first launch: show the main app window only;
  do not add onboarding, a setup prompt, or automatically open Settings. At the
  time, control availability without a TV was undecided. TV discovery remains
  available from TV settings when the user opens it. Updated the
  [information architecture](../planned-information-architecture.md),
  [GUI plan](../ux-gui.md), and software architecture.
- 2026-10-05 — Owner added a Main Toolbar at the bottom of the Sidebar with a
  Settings button that opens a Settings Window. TV is first in the Settings
  Sidebar. TV settings lists Saved TVs and Discovered TVs in a radio-button
  table above Discover TVs; hide the table when empty and center the button,
  preselect the table's only row, and require a choice when multiple TVs are
  available. A Discovered TV becomes
  active and is saved after TV Identity Confirmation and pairing. At launch,
  resume the selected Saved TV and attempt to reconnect. Updated the
  [UX glossary](../ux-term.md),
  [information architecture](../planned-information-architecture.md), and
  [GUI plan](../ux-gui.md).
- 2026-10-05 — Owner started P1-M3 to define canonical UX terms, information
  architecture, and the planned GUI before implementation. The app opens to
  the Remote View; remaining Sidebar contents are TBD. The volume slider is
  interactive when the selected TV supports exact-level control. See the
  [UX glossary](../ux-term.md),
  [information architecture](../planned-information-architecture.md), and
  [GUI plan](../ux-gui.md).
- 2026-10-05 — Owner added local-network TV discovery to the first usable
  app. Keep manual host entry as a fallback; choose and validate the discovery
  mechanism against the target TV. Updated the [software architecture](../architecture.md)
  and [repository map](../planned-repository-architecture.md).
- 2026-10-05 — Owner selected Iced for the macOS window and manual host entry
  as the required add-device path. The first usable app should include core
  controls, wake, sources, installed-app launch, and text input where the TV
  supports them, with an ordinary-key response target under 150 ms on the
  owner's home network. Initial installation is for the owner's Mac and
  compatibility testing starts with KU75UA8090FXKR. The port-8001 policy
  remains open until that TV is tested. The [software architecture](../architecture.md)
  and [repository map](../planned-repository-architecture.md) reflect these calls.
- 2026-10-05 — P1-M1 completed. Added the minimal `samsung-tv-remote` binary
  package and Hello World entry point requested for the Rust starter. The
  gate passed: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D
  warnings`, and `cargo test`; `cargo run --quiet` printed `Hello, world!`.
  Updated P1-M1 to Done. P1-M2 decisions and remaining hardware questions are
  recorded in the entries above and below.
- 2026-10-05 — Research handoff prepared for P1-M2: target recorded as a
  MacBook Pro M5 Max (128 GB), macOS Tahoe 26.7.1, and Samsung KU75UA8090FXKR;
  Samsung identifies its product family as Tizen. Added minimum remote and
  failure contract, macOS local-network/privacy and distribution constraints,
  and token/logging rules. Exact endpoint, TLS, pairing, key, and wake behavior
  remain hardware questions. Research notes:
  [summary](../research/p1-m1-initial-research.md),
  [protocol](../research/samsung-tv-remote-protocol.md),
  [security](../research/samsung-tv-protocol-security.md).
- 2026-10-05 — Phase 1 planned with P1-M1 Initial Research and P1-M2 Initial
  Architecture.
- 2026-10-06 — P1-M4 completed. Added keyboard focus support via Iced's default
  focusable widget behavior, idempotent Settings window handling, and preserved
  navigation state on window close. All sidebar navigation items route correctly
  to their views without triggering network or device operations. The Global
  Messages Pane implements scroll-follow behavior and resizable height. All
  validation gates passed: `cargo fmt --all`, `cargo clippy --all-targets -- -D
  warnings`, and `cargo test` (6 tests). No network or storage services present.
