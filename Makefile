.PHONY: run build bundle clean test fmt check clippy install uninstall
.DEFAULT_GOAL := bundle

# Binary name from Cargo.toml
BINARY := samsung-tv-remote

run:
	cargo run

build:
	cargo build --release

bundle:
	@set -eu; \
	if [ -n "$${SAMSUNG_TV_CODESIGN_IDENTITY:-}" ]; then \
		zsh scripts/build_macos_bundle.sh; \
	elif [ -s target/local-signing-identity.txt ]; then \
		IFS= read -r samsung_signing_identity < target/local-signing-identity.txt; \
		SAMSUNG_TV_CODESIGN_IDENTITY="$$samsung_signing_identity" zsh scripts/build_macos_bundle.sh; \
	else \
		printf '%s\n' 'No signing identity configured. Run zsh scripts/create_local_signing_identity.sh or set SAMSUNG_TV_CODESIGN_IDENTITY.' >&2; \
		exit 2; \
	fi

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
