# P1-M10: First Live TV Connection and Control

Status: Archived

## Architecture

Keep the existing dependency direction: `domain` defines semantic actions;
`application` owns selection, trust/pair/connect policy, dispatch, and results;
`infrastructure` owns Samsung frames, sockets, local discovery, storage, and
macOS services; `presentation::iced` projects application facts. `main.rs`
wires the concrete adapters. Neither a widget nor a discovered address can
send a raw `KEY_*` value.

```text
Settings candidate/manual host -> confirm identity/trust -> Pairing
    -> save DeviceId + non-secret record; token in Keychain
    -> select generation -> connect one Samsung session

Remote button -> SendRemoteAction(target, generation, action)
    -> current snapshot policy -> bounded dispatcher -> mapped key/frame
    -> serial session write -> typed result/event -> UI projection
```

## Contracts and ownership

| Owner | M10 responsibility |
| --- | --- |
| `domain` | Keep `DeviceId` opaque and `RemoteAction` finite; add only the device/trust display values needed by application policy. |
| `application::device_service` and ports | Probe/confirm candidate, Pairing, save/select, reconnect, re-pair, forget, and typed failure mapping. Own which host may receive an existing token. |
| `application::state` | Extend the current generation-scoped Pairing and Connection facts from actual use-case/session events. Distinguish permission denied, trust changed, token rejected, timeout, and offline recovery where needed. |
| `application::dispatcher` | Sole bounded FIFO for current-device remote requests, request IDs, admission, backpressure, cancellation, and one terminal result per admitted request. Re-evaluate policy when dequeuing. |
| `application::monitoring` | Atomic snapshot plus revisioned events and a bounded terminal-result journal, so observer restart cannot lose or duplicate outcomes. |
| `infrastructure::samsung` | Bounded codec, explicit action-to-key map, one socket owner, serial writes, safe close/error events. No product retry or UI copy. |
| Other infrastructure | Bounded discovery/probing, non-secret preferences, Keychain tokens, per-device certificate trust, and macOS local-network error interpretation. |
| `presentation::iced` | Settings setup and recovery, Remote View dispatch, disabled reasons, app-wide messages and Activity projection; no sockets or token handling. |

Use narrow async ports (`TvProbe`, `TvSession`, `DeviceDiscovery`,
`DeviceRepository`, `SecretStore`, and `TrustStore`) with typed errors. A
`DeviceId` is a generated, collision-resistant local record ID, not proof of
TV identity. Persist its mapping with the selected TV; never reconstruct it
from name, IP, or MAC alone. A changed host must pass the saved trust check
before any existing token is supplied. Discovery yields untrusted candidates.
Accept only a host or IP input, not a URL with arbitrary scheme/path/port;
resolve hostnames and check the resolved endpoint on every connect and
reconnect. Bound targets to local/private or link-local addresses, reject
redirects to arbitrary Internet hosts, and do not allow DNS changes to bypass
saved trust. Restrict the current pure lifecycle setters so only application
use cases can mark a live Pairing or Connection ready in production.

## Connection and trust state machine

1. **Candidate:** Discovery or manual entry gives only an untrusted host and
   display hints. A probe has a timeout and bounded response; it cannot mark
   Pairing or Connection ready.
2. **Confirmation:** Present observed name/model/address and client name.
   For an untrusted certificate, present its fingerprint with an explicit
   per-device trust decision. Never install a global insecure TLS verifier.
3. **Pairing:** Open the verified or explicitly pinned secure channel without
   a token and wait for the TV-side consent event. Save a returned token only
   after acceptance. Commit the device record, trust record, and Keychain
   token as one logical setup operation with cleanup on partial failure.
   Denial, timeout, malformed event, Keychain failure, or user cancellation
   leaves no active paired TV. Re-pair must never send the old token to a
   changed identity.
4. **Connected:** A successful authenticated channel-open event and live
   socket establish Connection readiness for the current selection generation.
   A stored token alone does not. A close, failed heartbeat, or read timeout
   clears readiness promptly.
5. **Recovery:** Bounded reconnect applies only to the same trusted target.
   Token rejection stops and requests re-pair. Pin or identity change stops and
   requests confirmation. A timeout/offline state offers retry without
   silently changing host, port security, or selected TV.

Use secure port 8002 only. Confirm the target's certificate behavior before
choosing the pin implementation. No port-8001 path or TLS-to-plaintext
downgrade belongs in M10. If 8002 cannot support Pairing and reconnect on the
target, gate 1 records that the target cannot satisfy M10's live acceptance
under the owner's secure-only decision. Pairing and Connection attempts carry
both selection generation and an attempt ID, so two attempts in the same
generation cannot let an older completion overwrite a newer one.
If macOS denies local-network access while its first alert is pending, show a
recoverable state and allow a bounded retry after approval. Never loop on
permission denial or token rejection.

Adding a candidate while another TV is selected may open a temporary
pairing-only session. Its Pairing progress is provisional setup state, separate
from the current Selected TV's lifecycle, and it cannot accept remote actions.
Keep the previous TV selected until the new Pairing and credential save
succeed. Then cancel its work, close its session, advance the selection
generation, and either transfer
the candidate session to the sole active-session owner or close it before
reconnecting. A failed candidate leaves the previous selection intact.

## Dispatch and result semantics

P1-M9's `Eligible` is a pure policy result. M10 turns it into a separate
dispatcher admission result: `Queued(request_id)` or a typed rejection such
as stale selection, pairing required, not connected, unsupported action, or
busy. The dispatcher reads one authoritative snapshot for admission and
checks the target, generation, capability, and session again before write.
The queue has a documented small capacity; full means an immediate visible
busy result, never silent loss or unbounded memory growth.

Replace P1-M9's global `control_status()` projection, which currently tests
`PowerToggle` as a stand-in for every button, with per-action availability.
Otherwise the deferred Power Toggle would disable working navigation or an
eligible navigation action could incorrectly enable Power Toggle. The Iced
button and application admission for each action must share that decision.

Only the dispatcher owns request order. The session owns serial socket
writes, with no second application-level queue or independent retry loop.
Switching or forgetting the selected TV cancels pending work and closes its
session. A late old-session event remains attributable to its original
request but cannot update the current selected TV. Non-idempotent actions are
never replayed after an ambiguous write.

For each queued request, record exactly one terminal result:

- **Not sent:** cancelled before write, unavailable, unsupported, or a
  definite pre-write failure.
- **Written:** the outbound WebSocket frame was fully flushed by the local
  transport; this does not prove the TV acted on it, even if the session later
  disconnects.
- **Uncertain:** the transport cannot determine whether a complete frame was
  written, such as a partial write followed by disconnect; never auto-retry
  that action.

The monitoring snapshot carries the selected generation, Pairing/Connection,
pending request IDs, and unacknowledged terminal results. Revisioned events
let Iced resync after subscription gaps. A bounded result journal applies
backpressure before admitting new work if it cannot retain another result.
Global Messages summarizes user-relevant events; Activity shows structured
recent requests and Connection events. Neither publishes protocol frames or
private device data.

## Protocol and UI boundaries

The Samsung adapter accepts only `RemoteAction` and maps it to validated
`KEY_*` values. Encode one bounded `ms.remote.control` Click frame per press;
parse only expected bounded connection/authorization events. Keep the key map
table and model-specific exceptions here. Record a compatibility profile for
the tested model/firmware from hardware evidence; unknown support does not
become supported from a successful socket write or model name alone. Reject
`PowerToggle` as deferred even when Pairing and Connection are ready, and keep
its visible button disabled. The volume slider continues to have no action.

Start the profile with the research map, then verify each M10 action on
hardware: arrows to `KEY_UP`, `KEY_DOWN`, `KEY_LEFT`, `KEY_RIGHT`; Select to
`KEY_ENTER`; Back to `KEY_RETURN`; Home to `KEY_HOME`; Mute to `KEY_MUTE`;
Volume Up/Down to `KEY_VOLUP`/`KEY_VOLDOWN`. Keep these strings only inside
the Samsung adapter. See the [protocol research](../research/samsung-tv-remote-protocol.md).

The Settings Window gains candidate rows, manual entry, confirmation,
Pairing, retry, re-pair, and forget states. A sole discovered row may be
provisionally checked as planned, but it becomes the active Selected TV only
after confirmation and Pairing. The Remote View keeps the existing layout and
shows separate Pairing and Connection states, safe recovery guidance, and the
latest request outcome. Both windows observe the same application coordinator.
Network tasks and subscriptions must not own or recreate the socket when a
window closes.
Map defined remote keyboard shortcuts and button presses through the same
typed intent. Suppress remote shortcuts while a text field or modal owns the
key and ignore accidental repeat for non-repeatable actions.

Build the app bundle with a stable identifier, privacy text, and a stable
local self-signed identity. Add `NSBonjourServices` if Bonjour discovery is
selected. Native review must use that bundle because macOS may treat Terminal
child processes differently for local-network permission. Apple recommends an
Apple-issued identity for reliable permission tracking across rebuilds;
the owner does not have one, so verify the locally signed bundle on the target
Mac and record any permission-state limitation across rebuilds. Test the first
prompt in a fresh macOS user account or VM instead of changing the stable
bundle ID. If local signing cannot yield a working permission flow, record
that M10 cannot pass native acceptance under the current signing choice.
Never commit signing material. See
[Apple's local-network privacy guidance](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy).

## Deterministic tests and hardware evidence

- Unit fixtures: target validation, exact frame encoding, bounded event
  parsing, key map, token redaction, and transition rules.
- Fake ports/server: first-time consent/denial, saved-token reconnect,
  revocation, TLS pin change, timeout, malformed and oversized frames,
  connection drop, and rejection of port-8001/plaintext attempts.
- Dispatcher scenarios: FIFO order, capacity, queue full, selection switch,
  forget, cancel before write, uncertain partial write, exactly one terminal
  result, and observer detach/reattach with snapshot resync.
- UI scenarios: unavailable controls and reasons, setup flow, cross-window
  messages, request attribution, per-action availability with Power Toggle
  disabled, and no claimed TV state from a write.
- Native hardware: follow the [overview's live acceptance script](milestone-10-overview.md)
  on the powered-on target and keep a sanitized compatibility record.

Dependency/API details are selected at gate 1 after inspecting the target and
current library documentation. No test may depend on a real TV or a particular
home network. Use the [contribution guide](../../contribution-guide.md) gates at
completion.
