.PHONY: run build clean test fmt check clippy install uninstall
.DEFAULT_GOAL := run

# Binary name from Cargo.toml
BINARY := samsung-tv-remote

run:
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
	install -d /usr/local/bin
	install -m 755 target/release/$(BINARY) /usr/local/bin/$(BINARY)

uninstall:
	rm -f /usr/local/bin/$(BINARY)
