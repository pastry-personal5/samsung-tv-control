# P1-M1: Initial Research Architecture

Status: Archived

## Approach

This milestone produces decision inputs rather than application code. Keep
findings in small, attributable notes and separate confirmed facts from
assumptions. Use sanitized fixtures or protocol descriptions when describing
TV communication; do not retain pairing tokens, device IP addresses, or raw
local-network captures.

The consolidated handoff is
[P1-M1 Initial product and platform research](../research/p1-m1-initial-research.md),
with protocol and threat detail in the linked research notes. The decision
record left the UI bridge, packaging route, discovery method, TLS behavior, and
support boundary for P1-M2 and hardware validation. P1-M2 has since recorded
the architecture decisions; target-specific behavior remains a
hardware-validation item.

## Outputs

- A compatibility and protocol-risk summary.
- A shortlist of macOS UI and Rust integration constraints.
- An open-decisions list for P1-M2.

## Sequencing

Complete the research summary before selecting the initial architecture in
P1-M2. Amend this plan if research reveals a compatibility or security blocker.
