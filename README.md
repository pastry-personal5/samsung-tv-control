# Samsung TV Remote

Super-fast remote controller for macOS, written in Rust.

The app supports discovery or manual address entry, certificate checking,
TV-approved pairing, saved-TV reconnect, remote keys, and Wake-on-LAN. Power,
Remote, and Settings are functional; Sources, Apps, and Text Input are
placeholders. The visual remote supports pointer input and mapped keyboard
shortcuts. Its Play/Pause control sends `KEY_PLAY` when playback is unknown or
last requested paused, then `KEY_PAUSE` after a confirmed Play write.

Phase 1 is [active](docs/phase-1/phase-1.md). Wake on a physical TV and parts
of the native visual and accessibility review remain open. See the
[documentation index](docs/README.md) for current guidance and archived plans.

For setup, development, and validation, use the
[contribution guide](docs/contribution-guide.md). Do not commit pairing tokens,
TV addresses, or local-network captures.
