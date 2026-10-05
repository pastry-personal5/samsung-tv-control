.PHONY: run build clean test fmt check
.DEFAULT_GOAL := run

# Binary name from Cargo.toml
BINARY := samsung-tv-remote

run: build
	cargo run

build:
	cargo build --release

test:
	cargo test

check:
	cargo check

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean

install: build
	install -D target/release/$(BINARY) /usr/local/bin/$(BINARY)

uninstall:
	rm -f /usr/local/bin/$(BINARY)
