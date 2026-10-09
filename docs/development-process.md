# Development process

Status: Active

The [roadmap](roadmap.md) identifies the active phase. Each phase has stable
`P<phase>-M<milestone>` IDs; never renumber a milestone. Work in milestone order
unless the phase plan records a reason to proceed while acceptance is deferred.

| Item | Status values |
| --- | --- |
| Phase | `Planned`, `Active`, `Done` |
| Milestone | `Planned`, `Active`, `Done`, `Dropped` |
| Supporting document | `Draft`, `Proposal`, `Active`, `Archived` |

A milestone is `Done` when its checklist has acceptance evidence, the
[contribution-guide checks](contribution-guide.md) pass, and affected docs are
updated. A phase is `Done` when its exit criteria hold and every milestone is
`Done` or `Dropped`. Record deferred verification explicitly; implementation
alone does not make a milestone `Done`.

## Files and ownership

- `docs/roadmap.md`: phase status and link to each phase plan.
- `docs/phase-N/phase-N.md`: goal, exit criteria, milestone status, and links.
- `docs/phase-N/milestone-NN-overview.md`: goal, scope, acceptance checklist.
- `docs/phase-N/milestone-NN-architecture.md`: technical approach.
- `docs/phase-N/changelog.md`: dated decisions and design changes, newest first.
- `docs/architecture.md` and `docs/ux-*.md`: current implementation guidance.

Write the overview and architecture before implementing a new milestone. Keep
the phase plan brief and place detail in those two files. Commit messages use
the [contribution guide](contribution-guide.md); include milestone IDs in PR
titles and commit bodies when applicable.

## Archiving

Move completed milestone plans, superseded designs, and incorporated research
to `docs/archive/`, preserving their relative folder structure and links.
The phase plan continues to link to archived milestone records. Mark archived
files clearly and update current references to point to active documents.
Proposals remain proposals until an owner decision is recorded in the phase
changelog and the active design docs.
