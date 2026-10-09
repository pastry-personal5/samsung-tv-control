# P1-M1: Initial Research

Status: Archived

## Goal

Establish a concise evidence base for the first version of the macOS Rust
remote.

## Scope

In scope:

- Compatible Samsung TV families, local discovery, connection, and pairing
  behavior.
- Required remote actions and expected responsiveness.
- macOS distribution, permissions, and privacy constraints.
- Security risks for pairing tokens and local-network information.

Out of scope:

- Implementing the client, user interface, or production device discovery.
- Committing device credentials, test-TV data, or local-network captures.

## Completion checklist

- [x] Record sources, assumptions, and open questions in the
  [initial research summary](../research/p1-m1-initial-research.md) and linked
  protocol notes.
- [x] Identify the supported initial device and macOS targets in the research
  summary; exact TV endpoint behavior remains a hardware-validation item.
- [x] Define the minimum remote-control feature set and failure behavior in
  the research summary.
- [x] Document pairing-token storage and logging requirements in the
  [security research](../research/samsung-tv-protocol-security.md).
- [x] Summarize findings and decisions for P1-M2 in the phase changelog.

The documentation checklist and required Cargo gate are complete. The minimal
Hello World binary is the initial package scaffold; product implementation
remains out of scope for this milestone. Its initial minimum feature list was
later expanded by the owner decision recorded in the
[phase changelog](../../phase-1/changelog.md) and specified in the
[software architecture](../initial-software-architecture.md).
