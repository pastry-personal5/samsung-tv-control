# Phase 1 Changelog

Status: Archived

## Entries

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
