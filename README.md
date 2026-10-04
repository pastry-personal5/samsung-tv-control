# Samsung TV Remote

Samsung TV Remote is a super-fast native macOS remote application written in
Rust. It is intended to provide responsive, keyboard-friendly control of
compatible Samsung TVs on the local network.

## Status

The repository is being initialized. The Rust crate, macOS interface, and
device protocol implementation have not been committed yet.

## Development

Once `Cargo.toml` is present, use the standard Rust workflow:

```sh
cargo run
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

See [AGENTS.md](AGENTS.md) for repository layout, code style, testing, and
contribution guidance. Do not commit Samsung pairing tokens, device IP
addresses, or other local-network details.
