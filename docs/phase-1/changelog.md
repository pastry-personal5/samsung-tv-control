# Phase 1 Changelog

Status: Active

## Entries

- 2026-10-10 — Completed P1-M14. The signed bundle verified successfully and
  native review covered all primary views, Settings Discovery, Power Controls,
  and selected-row TV actions at the minimum window size. Keyboard focus and
  screen-reader exposure remain documented Iced/macOS product follow-ups; live
  playback state remains inferred pending a supported monitoring integration.

- 2026-10-10 — Classified "Remote request queued." as Debug. The Global
  Messages Pane excludes Debug diagnostics and does not count them as unread.

- 2026-10-10 — Moved Power Controls above Wake Steps. Moved Retry Connection
  and Forget Selected TV into the selected Saved TV row and removed the
  redundant Connection recovery card and current-selection text.

- 2026-10-10 — Recorded Tizen TV state-monitoring research. The local remote
  channel remains a command path with inferred playback state; documented
  observation options require a participating Tizen app, SmartThings capability,
  or a commissioned Matter endpoint.

- 2026-10-10 — Changed the visual remote's Play/Pause dispatch to send
  `KEY_PLAY` when playback is unknown or inferred paused and `KEY_PAUSE` after
  a confirmed Play write. A selected or reconnected TV returns to unknown;
  Samsung's remote channel does not provide playback-state observation.

- 2026-10-10 — Consolidated current documentation: shortened the root guides,
  made software architecture the current module reference, archived completed
  M1–M12 plans and superseded research/designs, and repaired relative links.
  Kept M13 Active because physical Wake acceptance is still deferred.

- 2026-10-10 — Consolidated the canonical UX terms, information architecture,
  and GUI specification around the implemented P1-M14 owner decisions. Moved
  the information architecture to its canonical UX filename and updated its
  links. The documents now distinguish the current visual remote, explicit
  Check TV → Pair path, separate Wake configuration, and current accessibility
  limits from proposed changes. Added a separate research note with
  discovery, pairing, and Wake interaction recommendations.

- 2026-10-09 — Owner removed the duplicate “Keyboard controls” panel below
  the P1-M14 visual remote. Remote View now shows the image alone; existing
  shortcuts remain for mapped actions. The image's pointer regions, including
  Play/Pause, have no individual keyboard focus or screen-reader labels.
  Updated the milestone plan and UX documents with that limitation. Cargo
  gates and the signed bundle build pass; a native Remote View inspection
  confirms the panel is absent.

- 2026-10-09 — Removed routine “Opened Settings” and “Closed Settings”
  information entries from Global Messages. Settings window lifecycle and
  existing messages are preserved.

- 2026-10-09 — Reviewed the implemented P1-M14 visual remote against the
  owner's revised direction. The bundled remote image keeps its pointer hit
  regions; hover highlights usable regions and unavailable regions are dimmed.
  Added focusable, named companion buttons with the same typed actions and
  disabled guidance because canvas regions are not separate keyboard targets.
  Centered the titled primary-view boxes and removed stale per-key verification
  guidance. At that time, the owner chose `KEY_PLAY` for the new Play/Pause
  face; pause behavior remained unverified on hardware. Updated P1-M14 and UX
  documents to reflect the visual remote and that limit. The signed bundle
  built and its Remote View and companion controls were visually inspected at
  the minimum window size; Power View was also inspected. Focus, Settings and
  placeholder views, and screen-reader labels remain unverified.

- 2026-10-09 — Revised the P1-M14 plan using the owner's VS Code Sidebar
  reference. The Main Window Sidebar becomes a compact bitmap-led rail with
  edge-to-edge, gapless button slots, a Sidebar-matched resting background,
  and separate hover/selected/focus states. Each Main Pane view gains one
  horizontally centered box titled with the view name; Remote and Power
  controls remain grouped within it. Original bitmap artwork is planned for
  Sidebar and implemented Main Pane buttons, with visible text retained where
  icons alone would be ambiguous. Added an Iced/bundle feasibility gate,
  accessibility and fallback rules, and native visual acceptance criteria.
  This is a plan revision, not an implementation.

- 2026-10-09 — Reviewed and hardened P1-M13. A selected TV now remains in
  the in-memory Wake list if the full list load fails, and saving Wake settings
  repairs a missing cache entry. Invalid active-interface configuration is
  rejected before it can cancel an in-progress send. Wake readiness waits now
  stop promptly after cancellation, even during an in-flight connection, and
  continue to honor the 30-second deadline. The Iced control projection is
  refreshed when the short probe falls through to Wake and when reconnect
  fails. Added deterministic tests for these recovery paths and the selected
  active MAC being handed to the transport exactly once. Cargo checks pass;
  live-TV Wake remains unverified.

- 2026-10-09 — Implemented P1-M13 with a Power Sidebar item and Power View,
  per-TV wired/Wi-Fi Wake MAC settings, a single local-route directed-broadcast
  magic packet, bounded trusted reconnect, cancellation and stale-result
  protection, and connected-only Samsung `KEY_POWER`. The Remote View retains
  Power Toggle: its disconnected path moves to Power View, checks the saved
  connection for about two seconds, and Wakes only after an unreachable or
  timed-out result. Missing Wake configuration disables the disconnected
  button; pairing failures stop without a packet. Cancellation distinguishes
  pre-send, unknown send outcome, and post-send status. The signed native
  bundle was inspected for Sidebar order and Power View. Cargo gates pass;
  physical Wake, local-network permission recovery, and latency remain
  unverified.

- 2026-10-09 — Completed P1-M12. The main and Settings windows now enforce
  1100 x 760 and 1000 x 660 minimum sizes and always use Iced's dark theme.
  The Sidebar has no product/title or shortcut chrome and uses an icon-only
  Settings control with a Settings tooltip and ⌘, shortcut. The Remote View
  no longer duplicates selected-TV, Pairing, Connection, or aggregate
  availability text. Activity View and its projection were removed. Global
  Messages now has a pointer-draggable split bar, a scrollable compact feed,
  fixed-width semantic Info/Warning column, and concise message text without
  sequence numbers, source bookkeeping, or an idle follow label. Manual key
  verification was removed end-to-end: all standard actions are eligible after
  selection, Pairing, and Connection; `verified_actions` is ignored when
  loading and omitted on save, including legacy `Select` values. Cargo format,
  Clippy, and test gates pass.

- 2026-10-09 — Refined P1-M12 with the owner: the main Sidebar will lose its
  product title, Main Toolbar label, and view-shortcut hint; Settings becomes
  an accessible icon-only control. The Remote View lifecycle summary and
  Activity View will be removed. A draggable split bar replaces the
  Global-Messages height slider and its resize shortcuts. Global Messages will
  be the single scrollable user-message surface, using a fixed-width labelled
  and semantic-colour severity column, smaller readable text, concise
  user-oriented content, no sequence/source bookkeeping, and no idle
  “Following new messages.” text. Added the associated canonical Settings and
  message terminology. No implementation has begun.

- 2026-10-09 — Planned P1-M12 through P1-M14 with the owner. P1-M12 adopts
  the always-dark graphite theme, main/settings minimum sizes of 1100 x 760
  and 1000 x 660, and removes manual per-key action verification. The standard
  action list is now implicitly verified; obsolete `verified_actions` data,
  including historical `Select`, will be ignored when records are read and
  removed on a later save. This deliberately supersedes the narrow P1-M11
  saved-action spelling break because that field no longer has product
  meaning. P1-M13 adds user-selected wired/Wi-Fi Wake MAC configuration, one
  immediate WoL packet, a 30-second readiness-only retry window, persistent
  Wake Steps, and connected-only `KEY_POWER` toggle behavior. P1-M14 refines
  Remote and Settings into high-contrast cards. No implementation has begun.

- 2026-10-09 — Completed P1-M11. Moved restoration, discovery/probe,
  Pairing/re-pair, reconnect, selection/session generations, dispatch,
  cancellation, and stale-event policy from Iced into
  `application::tv_control_coordinator`; `main.rs` now composes the concrete
  adapters. Added an application `tv_session` port and kept Samsung sockets,
  channels, frames, and `KEY_*` mappings in infrastructure. Renamed
  `RemoteAction::Select` to `Enter`; Samsung still sends `KEY_ENTER`. Renamed
  application modules `command` → `remote_request`, `state` → `control_state`,
  `dispatcher` → `remote_dispatcher`, `device` → `device_repository`,
  `device_service` → `tv_setup_service`, `discovery` → `tv_discovery`,
  `secret` → `credential_store`, `trust` → `certificate_trust`, and `target` →
  `tv_address`; renamed infrastructure `ssdp` → `ssdp_discovery`, `storage` →
  `preferences`, and moved `keychain` under `macos`; renamed Iced `message` →
  `ui_message`. Retained `device` for saved-record identity vocabulary and
  `Select` only in historical records, UI selection wording, and the legacy
  failure fixture. New `verified_actions` values serialize as `Enter`; old
  files containing `Select` are not read or migrated, but remain untouched
  when loading fails. README and M11 architecture document manual backup and
  re-pair. Keychain identities, credentials, certificate pins, bundle ID,
  and network behavior were not changed. Baseline and final Cargo format,
  Clippy, and test gates pass (92 → 97 tests: 80 unit and 17 integration in
  the final tree). No live-TV exercise was performed; M10's deferred hardware
  checks remain deferred.

- 2026-10-09 — Owner revised P1-M11's data decision: no compatibility is
  required for pre-M11 saved records or older app builds. Removed the planned
  `Select` decoder and automatic migration. The plan now requires an explicit
  storage error for an old file, a recoverable manual reset/re-pair procedure,
  and no automatic deletion of old preferences or credentials. This supersedes
  the earlier migration decision below; no P1-M11 code had changed yet.

- 2026-10-09 — Reviewed and revised P1-M11 with the owner. The target now
  extracts live session and dispatch orchestration from Iced into an
  application coordinator. The owner chose to migrate saved
  `verified_actions` values from `Select` to `Enter`, keep old records readable
  in the updated app, and waive rollback compatibility with older app builds.
  Added migration, boundary, and deterministic completion criteria to the
  plan. No P1-M11 implementation has started.

- 2026-10-09 — Owner marked P1-M10 Done. The milestone closes with the
  previously recorded hardware verification deferred, not passed: discovery
  fallback, native local-network permission recovery, re-pair/forget and
  failure/switching paths, remaining bundle review, and latency measurement.
  The under-150-ms target remains unverified. The sanitized hardware matrix
  remains the source of tested support claims.

- 2026-10-09 — Owner deferred the remaining P1-M10 human acceptance work
  without a revisit date: empty/denied/timed-out discovery fallback to manual
  address entry; signed-bundle local-network permission alert, denial recovery,
  and retry; re-pair, forget, failed Connection, and switching TVs during
  pending work; the remaining native-bundle review; and optional latency
  measurement. Empty-install setup and the sanitized hardware matrix remain
  recorded as completed evidence.

- 2026-10-09 — Owner reported successful human verification of the empty-install
  setup journey from the signed bundle: find or manually enter the powered-on
  TV, confirm it, complete Pairing, and reach a live Connection. Marked that
  M10 acceptance item complete; it does not verify the deferred recovery paths.

- 2026-10-09 — Owner reported a successful human discovery test from the
  signed bundle: the selected mechanism found a Samsung-like candidate. Marked
  the candidate-discovery acceptance item complete. Empty/denied/timed-out
  fallback, local-network permission recovery, and failure/switching checks
  remain open; discovery is not physical identity confirmation.

- 2026-10-09 — Planned P1-M11 as a behavior-preserving Clean Architecture
  refactoring milestone after P1-M10. It applies the canonical UX vocabulary,
  including the semantic `Select` to `Enter` rename, clarifies application
  ports and infrastructure adapters, and preserves storage, Keychain, queue,
  TLS, and live-control behavior. It does not replace M10's deferred hardware
  acceptance.

- 2026-10-08 — Owner verified signed-bundle Pairing, saved-token reconnect,
  Retry Connection after a temporary network interruption, and all ten
  on-screen keys on KU75UA8090FXKR (T-NKLAAKUC-2310.0, BT-S; TV on Ethernet).
  Activity reported all ten key requests as written to the TV connection, with
  the TV response unverified by the app; the owner reports the keys visibly
  working. M, Home, directional, minus, Enter, and Escape keyboard shortcuts
  worked; Shift+= initially failed in a bundle of uncertain age, then worked
  after the physical Equal-key fix and signed-bundle rebuild.
  The owner deferred discovery, local-network permission recovery, and
  failure/switching human checks without a date. No latency measurement is
  planned, so the under-150-ms target remains unverified. Added deterministic
  rollback, uncertain-write, certificate-pin, shortcut dispatch, and
  written-result tests. Discovery receive errors now surface as recoverable
  warnings. Aligned Activity and Settings control labels with the canonical UX
  terms. The owner approved the existing keyboard mappings as the canonical
  pairs in `docs/ux-term.md` and requires an interview before any new shortcut.
  Added and ran `make bundle` using the saved local signing identity;
  the signed bundle passed verification. Cargo formatting, Clippy, and tests
  pass (81 tests). M10 remains open.
- 2026-10-08 — P1-M10 implementation in progress. The owner's Terminal probe
  reached secure port 8002, negotiated TLS 1.3, received HTTP 200 metadata,
  and obtained a token after physical-TV approval; the diagnostic discarded
  that token. Added bounded codec, pinned TLS session, Keychain token storage,
  device-scoped trust and preference stores, SSDP discovery with manual entry,
  pairing/reconnect/forget flows, a bounded serial command dispatcher, and
  per-action UI availability. A locally self-signed bundle builds and passes
  `codesign --verify` without changing Keychain trust settings. Actual button
  responses, saved-token reconnect, local-network permission behavior, and
  native UI acceptance are still under test. No address, fingerprint, or real
  token was recorded in the repository.
- 2026-10-08 — Owner reviewed P1-M10 and chose secure port 8002 only, with no
  plaintext fallback; deferred Power Toggle to the wake/power milestone; and
  chose local signing because no Apple-issued identity is available. The owner
  can approve Pairing and observe button results during hardware acceptance.
  The Mac and TV share a LAN, and the address can be supplied privately if
  discovery fails. Tightened the plan's trust, session, dispatch, and macOS
  permission gates.
- 2026-10-08 — Planned P1-M10 as the first live-TV milestone and reopened
  Phase 1 for its end-to-end acceptance. The plan covers target-hardware
  decisions, trusted TV setup, Pairing, a live session, bounded dispatch,
  honest UI outcomes, and native validation on the already-on target TV.
  No live connection or protocol behavior is implemented by this plan.
- 2026-10-08 — Completed P1-M9 and Phase 1. Remote requests now retain the
  selection generation and are checked by a pure policy for selection, target,
  generation, pairing, and connection readiness. The policy’s eligible result
  is not a transport result; Iced enables controls only from the same policy
  and publishes no sent/confirmed message. Cargo format, Clippy, and test
  gates passed (40 tests), with no I/O added.
- 2026-10-08 — Completed P1-M8. Added independent, generation-scoped pairing
  and connection lifecycle facts with explicit ignored results for stale or
  absent selections. Changing selections resets lifecycle data; Iced renders
  separate safe labels and recovery guidance. Cargo format, Clippy, and test
  gates passed (38 tests), without a network adapter or background work.
- 2026-10-08 — Completed P1-M7. Application state now owns safe, in-memory
  selected-device display data and a monotonic selection generation. Pure
  select and clear transitions are idempotent for the current identity, and
  Iced displays the safe label while controls remain disabled. Cargo format,
  Clippy, and test gates passed (36 tests); no discovery, persistence, or
  network behavior was added.
- 2026-10-08 — Completed P1-M6 after reviewing the partial control gate. The
  application now rejects the original typed request with `NoSelectedTv`;
  the presentation reducer no longer manufactures a device ID or displays a
  debug-formatted outcome. Remote View derives its disabled reason from the
  application state. Native macOS review showed the same no-TV shell, and
  Cargo format, Clippy, and test gates passed (33 tests). Also corrected the
  stale README and macOS Makefile recipes.
- 2026-10-08 — Owner chose to preserve Iced and revise P1-M4's shell-level
  accessibility criterion to keyboard shortcuts and visible disabled reasons.
  Marked P1-M4 Done after native macOS visual review and the required Cargo
  gates passed (31 tests). Iced controls remain absent from the macOS
  accessibility tree; native screen-reader support is a product follow-up.
  Removed an earlier duplicate completion entry that incorrectly claimed
  Iced's default button and slider keyboard focus.
- 2026-10-08 — Reopened P1-M4 acceptance after a native macOS review. Added a
  route back to Remote, kept the lower panes visible when primary content
  overflows, tightened message scroll-follow detection, and guarded against a
  delayed Settings open event restoring a closed window. Added keyboard
  shortcuts for views, Settings, and pane height. The app launches; the routes,
  Settings lifecycle, and lower panes were checked in a native macOS session,
  including the minimum window size. Iced 0.14 does not expose these controls
  to the macOS accessibility tree in this build, so focus and accessible-name
  acceptance remain open.
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
  [milestone overview](../archive/phase-1/milestone-03-overview.md).
- 2026-10-05 — Planned P1-M4, Iced Application Shell and Navigation, as the
  next implementation milestone after P1-M3 completes. It builds the shared
  window and view shell without TV networking or device behavior. The plan
  uses one Iced application for the main and Settings windows and keeps
  presentation state separate from application commands. See the
  [overview](../archive/phase-1/milestone-04-overview.md) and
  [architecture](../archive/phase-1/milestone-04-architecture.md).
- 2026-10-05 — Owner added **Sources**, **Apps**, and **Text Input** as Sidebar
  destinations. Each opens Sources View, Apps View, or Text Input View in the
  Main Pane while the Global Messages Pane and Activity View remain available.
  Wake placement remains open. Updated the [UX glossary](../ux-term.md),
  [information architecture](../ux-information-architecture.md),
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
  [repository map](../archive/planned-repository-architecture.md),
  [UX glossary](../ux-term.md),
  [information architecture](../ux-information-architecture.md), and
  [GUI plan](../ux-gui.md).
- 2026-10-05 — Owner clarified first launch: show the main app window only;
  do not add onboarding, a setup prompt, or automatically open Settings. At the
  time, control availability without a TV was undecided. TV discovery remains
  available from TV settings when the user opens it. Updated the
  [information architecture](../ux-information-architecture.md),
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
  [information architecture](../ux-information-architecture.md), and
  [GUI plan](../ux-gui.md).
- 2026-10-05 — Owner started P1-M3 to define canonical UX terms, information
  architecture, and the planned GUI before implementation. The app opens to
  the Remote View; remaining Sidebar contents are TBD. The volume slider is
  interactive when the selected TV supports exact-level control. See the
  [UX glossary](../ux-term.md),
  [information architecture](../ux-information-architecture.md), and
  [GUI plan](../ux-gui.md).
- 2026-10-05 — Owner added local-network TV discovery to the first usable
  app. Keep manual host entry as a fallback; choose and validate the discovery
  mechanism against the target TV. Updated the [software architecture](../architecture.md)
  and [repository map](../archive/planned-repository-architecture.md).
- 2026-10-05 — Owner selected Iced for the macOS window and manual host entry
  as the required add-device path. The first usable app should include core
  controls, wake, sources, installed-app launch, and text input where the TV
  supports them, with an ordinary-key response target under 150 ms on the
  owner's home network. Initial installation is for the owner's Mac and
  compatibility testing starts with KU75UA8090FXKR. The port-8001 policy
  remains open until that TV is tested. The [software architecture](../architecture.md)
  and [repository map](../archive/planned-repository-architecture.md) reflect these calls.
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
  [summary](../archive/research/p1-m1-initial-research.md),
  [protocol](../archive/research/samsung-tv-remote-protocol.md),
  [security](../archive/research/samsung-tv-protocol-security.md).
- 2026-10-05 — Phase 1 planned with P1-M1 Initial Research and P1-M2 Initial
  Architecture.
