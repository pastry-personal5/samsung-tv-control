# Repository guidelines

Samsung TV Remote is a native macOS app in Rust. Keep application code in
`src/`, integration tests in `tests/`, bundled resources in `assets/`, and
documentation in `docs/`. Keep UI, application policy, Samsung protocol, and
macOS services in their respective modules; see [architecture](docs/architecture.md).

Use stable Rust, `rustfmt`, standard Rust naming, small public interfaces, and
explicit errors. Add comments for intent or non-obvious protocol decisions.
Add deterministic tests for new behavior and fixes; use fakes at network and
pairing boundaries instead of a real TV.

The [contribution guide](docs/contribution-guide.md) owns development commands,
validation, and commit and PR conventions. Run its checks before a PR, and
claim a command works only after running it successfully with committed
configuration. Keep tokens, TV addresses, and other local-network details out
of commits.
