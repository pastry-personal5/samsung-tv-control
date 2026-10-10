# Phase 1: Foundation and First Live Control

Status: Active

Goal: Establish the macOS app, trusted local control, and usable Power and
Remote views.

## Exit criteria

- Research, architecture, UX terms, and implementation boundaries are recorded.
- The app pairs with a TV over the secure local endpoint and sends supported
  remote requests with honest outcomes.
- Wake and GUI refinements pass their remaining acceptance checks.

## Milestones

Completed plans are retained in the [archive](../archive/phase-1/).

| ID | Milestone | Status | Record |
| --- | --- | --- | --- |
| P1-M1 | Initial Research | Done | [Overview](../archive/phase-1/milestone-01-overview.md) · [Architecture](../archive/phase-1/milestone-01-architecture.md) |
| P1-M2 | Initial Architecture | Done | [Overview](../archive/phase-1/milestone-02-overview.md) · [Architecture](../archive/phase-1/milestone-02-architecture.md) |
| P1-M3 | UX Terms, Information Architecture, and GUI | Done | [Overview](../archive/phase-1/milestone-03-overview.md) · [Architecture](../archive/phase-1/milestone-03-architecture.md) |
| P1-M4 | Iced Application Shell and Navigation | Done | [Overview](../archive/phase-1/milestone-04-overview.md) · [Architecture](../archive/phase-1/milestone-04-architecture.md) |
| P1-M5 | Typed Control Contracts | Done | [Overview](../archive/phase-1/milestone-05-overview.md) · [Architecture](../archive/phase-1/milestone-05-architecture.md) |
| P1-M6 | No-Selected-TV Control Gate | Done | [Overview](../archive/phase-1/milestone-06-overview.md) · [Architecture](../archive/phase-1/milestone-06-architecture.md) |
| P1-M7 | In-Memory TV Selection | Done | [Overview](../archive/phase-1/milestone-07-overview.md) · [Architecture](../archive/phase-1/milestone-07-architecture.md) |
| P1-M8 | Connection State Projection | Done | [Overview](../archive/phase-1/milestone-08-overview.md) · [Architecture](../archive/phase-1/milestone-08-architecture.md) |
| P1-M9 | Remote Command Admission Policy | Done | [Overview](../archive/phase-1/milestone-09-overview.md) · [Architecture](../archive/phase-1/milestone-09-architecture.md) |
| P1-M10 | First Live TV Connection and Control | Done | [Overview](../archive/phase-1/milestone-10-overview.md) · [Architecture](../archive/phase-1/milestone-10-architecture.md) · [Hardware matrix](../archive/phase-1/milestone-10-hardware-matrix.md) |
| P1-M11 | Clean Architecture Refactoring | Done | [Overview](../archive/phase-1/milestone-11-overview.md) · [Architecture](../archive/phase-1/milestone-11-architecture.md) |
| P1-M12 | Dark GUI and Focused Navigation | Done | [Overview](../archive/phase-1/milestone-12-overview.md) · [Architecture](../archive/phase-1/milestone-12-architecture.md) |
| P1-M13 | Wake-on-LAN and Toggle Power | Active | [Overview](milestone-13-overview.md) · [Architecture](milestone-13-architecture.md) |
| P1-M14 | Remote and Settings GUI Refinement | Done | [Overview](milestone-14-overview.md) · [Architecture](milestone-14-architecture.md) |

M13 is implemented while physical Wake, permission recovery, and latency
verification remain deferred. M14 is complete: native layout review and all
documented Cargo and bundle gates passed. Iced/macOS keyboard focus and
screen-reader exposure remain product follow-ups, and playback state remains
inferred until a supported monitoring integration is added. M10 was closed by
the owner with its deferred checks recorded in the hardware matrix. The
[changelog](changelog.md) preserves decisions and acceptance history.
