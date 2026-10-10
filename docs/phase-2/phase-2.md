# Phase 2: Conditional Local Control Surfaces

Status: Planned

## Goal

Turn the existing Sources, Apps, and Text Input placeholders into useful,
capability-gated local controls while preserving Phase 1's safety boundaries:
typed intents, serialized requests, pairing/connection gates, no secrets in
logs, and honest outcomes.

Phase 2 does not claim generic TV-state monitoring. Its local WebSocket path
can report a probe result or write outcome, not arbitrary final TV state.

## Product decision

Use the paired local remote connection as the only new transport in this
phase. It keeps the app private to the local network and avoids an account,
backend, OAuth redirect flow, public webhook, and cloud-token storage.

SmartThings, Matter, and a companion Tizen app remain future independent
providers. See the [control and monitoring capability map](../research/samsung-tizen-control-monitoring-capability-map.md).

## Milestones

| ID | Milestone | Outcome | Status |
| --- | --- | --- | --- |
| P2-M1 | Capability catalog and proof model | Per-TV capability probes/cache; one vocabulary distinguishing unavailable, supported, requested, and observed. | Planned |
| P2-M2 | Sources | Named, editable source actions with conservative local-key fallbacks and capability gating. | Planned |
| P2-M3 | Apps | Per-TV installed-app discovery, cached catalog, refresh, and conservative launch requests. | Planned |
| P2-M4 | Text Input | Session-bound native-IME text sending with privacy safeguards and explicit limits. | Planned |
| P2-M5 | Integrated resilience and accessibility | Reconnect/selection invalidation, saved-data migration, deterministic fakes, native review, and target-TV matrix. | Planned |

## Milestone contracts

### P2-M1: Capability catalog and proof model

- Add a domain/application model for feature capability that is separate from
  connection state and remote-action eligibility.
- Probe only when connected and cache results with the selected TV identity,
  firmware/version clue when available, timestamp, and an expiration policy.
- Use outcome terms deliberately: `unsupported`, `unavailable`, `requested`,
  `written`, and `observed` must not collapse into success.
- Invalidate ephemeral session capability on disconnect, re-pair, and TV
  selection change; retain persistent cached facts only with a clear stale
  state and explicit refresh.
- Do not parse unknown inbound WebSocket frames as state. Keep wire shapes and
  Samsung event names in infrastructure.

### P2-M2: Sources

- Implement the Sources view only for source actions whose local-key support
  is proven for the selected TV.
- Support user labels for source shortcuts; do not invent cable/device names.
- Prefer direct HDMI/tuner keys; offer the Sources menu as a manual-navigation
  fallback rather than scripting blind menu traversal.
- Show “Switching to HDMI 1…” / “Source request sent,” never “HDMI 1 active,”
  unless a future state provider has observed it.
- Test every offered input with connected hardware and after a wake/reconnect.

### P2-M3: Apps

- Fetch installed applications through the local channel, associate every app
  ID with one TV, and show the last refresh time/staleness.
- Launch only IDs returned by that TV. Treat launch as a request and surface
  a retry/refresh path for failures or stale entries.
- Start with ordinary app-home launches; exclude guessed IDs, content deep
  links, credentials, and search-text launches.
- Bound catalog growth and validate untrusted display names before rendering or
  storing them.

### P2-M4: Text Input

- Enable direct text only for a connected, active text-input session; provide
  clear empty/unsupported states.
- Encode original UTF-8 correctly, end sessions on completion/cancel/focus
  loss/disconnect, and retain no submitted text or Base64 payload in logs or
  persistence.
- Support Korean, mixed Unicode, symbols, emoji, and password-safe rendering
  in tests. Do not automatically submit a form.
- Document that OTT/custom on-screen keyboard grids need normal D-pad input;
  they are not automated in this phase.

### P2-M5: Integrated resilience and accessibility

- Verify selection changes, reconnects, re-pair, forgotten TVs, wake, stale
  results, cancellation, and bounded dispatcher behavior across all P2
  requests.
- Add deterministic protocol fakes for probes, catalog responses, IME events,
  send failures, malformed messages, timeout, and reconnect.
- Run the repository's required Rust gates and target-device matrix. Complete
  a native macOS review for keyboard access, focus, VoiceOver labels, disabled
  explanations, and the current minimum window size.

## Explicit non-goals

- SmartThings account linking, OAuth tokens, cloud backend, or webhooks.
- Matter commissioning.
- An owned companion Tizen TV app.
- Generic active-app, title, episode, source, panel-power, or playback-state
  monitoring.
- Streaming-app grid typing, blind remote macro playback, or app-specific
  deep links.
- Commercial-display RS-232/MDC control.

## Phase exit criteria

- Sources, Apps, and Text Input replace their placeholders only when the
  selected TV has the necessary proven capability.
- A command write is never displayed as an observed TV state.
- All local data is per-TV, stale-aware, removable, and excludes secrets and
  user-entered text.
- Automated tests use fakes rather than a live TV, and the target hardware
  matrix documents feature-by-feature evidence without sensitive data.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
  and `cargo test` pass before the phase is closed.

## Research basis

- [Capability map](../research/samsung-tizen-control-monitoring-capability-map.md)
- [Source changes and app launches](../research/source-changes-and-app-launches.md)
- [Text input from a macOS remote](../research/samsung-tv-text-input.md)
- [Tizen Samsung TV state monitoring](../research/tizen-tv-state-monitoring.md)
