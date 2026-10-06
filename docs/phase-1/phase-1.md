# Phase 1: Foundation and Architecture

Status: Active
Goal: Establish the validated product, technical, and UX direction required to
begin building the Rust macOS Samsung TV remote.

## Exit criteria

- Initial research records the supported-TV, connection, pairing, macOS, and
  privacy constraints that affect the product.
- Initial architecture identifies the proposed Rust crate structure, macOS UI
  boundary, device-communication boundary, and unresolved decisions.
- Canonical UX terms, information architecture, and the planned GUI are
  documented and ready to guide implementation.
- The presentation shell consumes typed application state and command contracts
  without importing Samsung protocol details or performing device I/O.

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

### P1-M5: Typed Control Contracts

Status: Done
Goal: Establish the small, pure domain and application command vocabulary that
the presentation layer will use for remote controls.
Plan: [overview](milestone-05-overview.md),
[architecture](milestone-05-architecture.md)
Notes: Implemented `DeviceId` (opaque identifier), `RemoteAction` (finite semantic enum), and `SendRemoteAction` (typed command). All validation gates pass.

### P1-M6: No-Selected-TV Control Gate

Status: Planned
Goal: Project an explicit no-selected-TV application state into the existing
shell so controls are disabled by policy rather than a presentation-only
placeholder.
Plan: [overview](milestone-06-overview.md),
[architecture](milestone-06-architecture.md)

### P1-M7: In-Memory TV Selection

Status: Planned
Goal: Let the application select and clear a known device in session state,
with a generation change that prevents stale work from crossing selections.
Plan: [overview](milestone-07-overview.md),
[architecture](milestone-07-architecture.md)

### P1-M8: Connection State Projection

Status: Planned
Goal: Represent pairing and connection readiness as distinct application
states and project them into the presentation layer without claiming live I/O.
Plan: [overview](milestone-08-overview.md),
[architecture](milestone-08-architecture.md)

### P1-M9: Remote Command Admission Policy

Status: Planned
Goal: Apply selected-device, pairing, connection, and known-action policy to
typed remote requests before any transport dispatch exists.
Plan: [overview](milestone-09-overview.md),
[architecture](milestone-09-architecture.md)
