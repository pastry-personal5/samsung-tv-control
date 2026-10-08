# Phase 1: Foundation and First Live Control

Status: Active
Goal: Establish the product and architecture foundations, then connect the
Rust macOS remote to its first live TV.

## Exit criteria

- Initial research records the supported-TV, connection, pairing, macOS, and
  privacy constraints that affect the product.
- Initial architecture identifies the proposed Rust crate structure, macOS UI
  boundary, device-communication boundary, and unresolved decisions.
- Canonical UX terms, information architecture, and the planned GUI are
  documented and ready to guide implementation.
- The presentation shell consumes typed application state and command contracts
  without importing Samsung protocol details or performing device I/O.
- The first live path connects to an already-on TV through confirmed trust and
  Pairing, sends supported remote actions, and reports honest request outcomes.

## Milestones

### P1-M1: Initial Research

Status: Done
Goal: Gather and record the constraints, risks, and requirements that shape the
first implementation.  
Plan: [overview](milestone-01-overview.md),
[architecture](milestone-01-architecture.md)
Notes: Research and platform constraints are summarized in
[initial product and platform research](../research/p1-m1-initial-research.md).

### P1-M2: Initial Architecture

Status: Done
Goal: Define the initial software architecture using the findings from P1-M1.  
Plan: [overview](milestone-02-overview.md),
[architecture](milestone-02-architecture.md)

### P1-M3: UX Terms, Information Architecture, and GUI

Status: Done
Goal: Define shared UX language, information hierarchy, and the first planned
GUI layout before implementation.
Plan: [overview](milestone-03-overview.md),
[architecture](milestone-03-architecture.md)

### P1-M4: Iced Application Shell and Navigation

Status: Done
Goal: Deliver a launchable Iced shell with the agreed navigation, main/settings
windows, and shared message and activity regions.
Plan: [overview](milestone-04-overview.md),
[architecture](milestone-04-architecture.md)
Notes: Native macOS acceptance verified the routes, Settings lifecycle, and
shared panes. The owner revised shell-level accessibility acceptance to
keyboard shortcuts and visible disabled reasons while retaining Iced; native
screen-reader support remains a product follow-up.

### P1-M5: Typed Control Contracts

Status: Done
Goal: Establish the small, pure domain and application command vocabulary that
the presentation layer will use for remote controls.
Plan: [overview](milestone-05-overview.md),
[architecture](milestone-05-architecture.md)
Notes: Implemented `DeviceId` (opaque identifier), `RemoteAction` (finite semantic enum), and `SendRemoteAction` (typed command). All validation gates pass.

### P1-M6: No-Selected-TV Control Gate

Status: Done
Goal: Project an explicit no-selected-TV application state into the existing
shell so controls are disabled by policy rather than a presentation-only
placeholder.
Plan: [overview](milestone-06-overview.md),
[architecture](milestone-06-architecture.md)
Notes: The application now rejects typed requests with their original target
preserved, and the shell projects the typed no-TV reason without a placeholder
device ID. All Cargo gates passed (33 tests).

### P1-M7: In-Memory TV Selection

Status: Done
Goal: Let the application select and clear a known device in session state,
with a generation change that prevents stale work from crossing selections.
Plan: [overview](milestone-07-overview.md),
[architecture](milestone-07-architecture.md)
Notes: Safe display data and a generation-scoped in-memory selection are now
projected into Iced; controls remain disabled until lifecycle facts exist.

### P1-M8: Connection State Projection

Status: Done
Goal: Represent pairing and connection readiness as distinct application
states and project them into the presentation layer without claiming live I/O.
Plan: [overview](milestone-08-overview.md),
[architecture](milestone-08-architecture.md)
Notes: Generation-scoped pure lifecycle updates now project independent pairing
and connection statuses; selection changes reset both facts.

### P1-M9: Remote Command Admission Policy

Status: Done
Goal: Apply selected-device, pairing, connection, and known-action policy to
typed remote requests before any transport dispatch exists.
Plan: [overview](milestone-09-overview.md),
[architecture](milestone-09-architecture.md)
Notes: A pure policy now returns typed rejections or an explicitly non-sending
eligible result; Iced availability derives from the same state.

### P1-M10: First Live TV Connection and Control

Status: In Progress
Goal: Connect the app to the owner's already powered-on TV and deliver the
first trusted, observable end-to-end remote-control path.
Plan: [overview](milestone-10-overview.md),
[architecture](milestone-10-architecture.md)
Notes: Includes hardware protocol decisions, TV setup and Pairing, trusted
persistence, live session, bounded dispatch, UI outcomes, and native acceptance.
The owner chose secure port 8002 only, deferred Power Toggle and wake, and
chose local signing for the test bundle. Initial secure endpoint and consent
probes passed. The owner verified signed-bundle Pairing, saved-token reconnect,
and all ten current keys. Discovery, native permission recovery, and
failure/switching checks have deferred human verification without a date.
The latency target remains unmeasured. Source/app/text features follow later.

### P1-M11: Clean Architecture Refactoring

Status: Planned
Goal: Improve the established live-control implementation's names, boundaries,
and test structure without changing behavior or persisted data.
Plan: [overview](milestone-11-overview.md),
[architecture](milestone-11-architecture.md)
