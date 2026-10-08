# Contribution Guide

Samsung TV Remote is a native macOS application written in Rust. This guide
defines the canonical local Cargo workflow for the current Rust package.

## Setup and development

Use the Rust toolchain selected by the repository (for example,
`rust-toolchain.toml` when present). Build and launch the debug application
with:

```sh
cargo run
```

The current package launches the Iced GUI. Settings can probe and pair a TV
over secure port 8002; the hardware acceptance run for P1-M10 is in progress.
For local-network permission testing, build the signed app bundle with
`make bundle`. It uses `SAMSUNG_TV_CODESIGN_IDENTITY` if set, or the identity
saved in `target/local-signing-identity.txt` by
`zsh scripts/create_local_signing_identity.sh`. The bundle is written to
`target/bundle/Samsung TV Remote.app`. Launch that bundle for native review;
`cargo run` does not prove native bundle permission behavior.

Run individual tests while iterating with `cargo test <name>`. Do not require a
physical TV or a local network for automated tests; use deterministic fakes for
protocol and pairing boundaries.

## Required validation

Before opening a pull request, run the full checks from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Format with `cargo fmt` when the formatting check fails. Address Clippy
warnings rather than suppressing them without a documented reason. `cargo test`
runs the package test suite; add or update tests for each behavior change and
bug fix.

## Commit messages

Write commit messages in [Conventional Commits](https://www.conventionalcommits.org)
(Angular) style.

- **Title:** `<type>[(scope)]: <summary>` — lowercase, imperative, 50 characters
  or fewer, no trailing period. Types: `feat`, `fix`, `docs`, `style`,
   `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.
- **Body** (optional): one blank line after the title, then short paragraphs
  explaining *why* the change was made, wrapped at 72 characters. Keep it brief.

Keep each commit focused. Example:

    feat(discovery): add mDNS TV discovery

    Find TVs on the local network so the remote can list devices without manual
    IP entry.

## Pull requests

Keep a pull request focused and explain the user-visible effect, relevant
protocol or UI decisions, and every validation command run. Include a
screenshot for macOS UI changes and sanitized logs for connection or pairing
changes. Never commit TV pairing tokens, device IP addresses, credentials, or
other local-network details.
