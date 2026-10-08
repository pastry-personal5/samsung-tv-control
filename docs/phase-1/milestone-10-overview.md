# P1-M10: First Live TV Connection and Control

Status: In Progress

## Goal

Connect the macOS app to the owner's already powered-on Samsung TV and send
the Remote View's supported button actions through a trusted, observable local
session. A user can add the TV, approve pairing on the TV, reconnect later,
and see whether each request was rejected, queued, written, or left uncertain.

This is one large end-to-end milestone with internal gates. It starts from the
P1-M9 policy-only shell; no current lifecycle value proves a live connection.
The first hardware target is the recorded KU75UA8090FXKR. The owner's
Terminal diagnostic confirmed the secure endpoint and consent flow. The owner
reports successful empty-install setup, Pairing, all ten required keys, and
successful discovery from the signed bundle. The owner also reports
saved-token reconnect and Retry Connection working. Native bundle permission
recovery and failure/switching paths have deferred human verification and
remain open acceptance items.
The owner tested on-screen controls and keyboard shortcuts, including Enter,
Escape, and Shift+= for Volume Up. Activity reported all ten key requests as
written to the TV connection, with the TV response unverified by the app.
The under-150-ms response target is unmeasured, with no measurement planned.
The owner confirmed the target Mac and TV are on the same local network and
can provide the TV address privately during implementation if discovery fails.

## User journey

1. Open Settings > TV and discover candidates, or choose **Enter TV Address**
   when discovery fails or finds nothing. Validate a local address and probe
   the TV without treating discovery or metadata as proof of identity.
2. Review the TV name/model/address and the stable client name. Confirm the TV
   identity and, if needed, its certificate fingerprint. The displayed
   metadata and fingerprint are observations, not proof of identity; confirm
   the matching approval prompt on the intended physical TV.
3. Save the selected TV after successful trust and Pairing. Return to Remote
   View with separate Pairing and Connection states. Controls become available
   only after the current selected generation has a usable live session.
4. Press an ordinary navigation or volume button. The app admits the typed
   request, writes one mapped key through the active session, and reports the
   transport result without claiming the TV changed state merely because a
   frame was written.
5. Restart the app. It restores the most recently selected saved TV, attempts
   a bounded reconnect with its device-scoped credential, and shows an
   actionable state if the TV, permission, token, or trust has changed.

## Scope

- Probe the powered-on target's 8002 secure endpoint, metadata, TLS
  certificate, consent/token event, and core key behavior before committing
  compatibility claims. Record only sanitized findings.
- Add bounded TV discovery with manual local-address entry as a complete
  fallback. Identify candidates in Settings; never save or select an
  unconfirmed discovery result as a trusted TV.
- Add device records and persistence for non-secret information, a
  device-scoped Keychain token store, and device-scoped certificate trust.
  Support explicit re-pair and forget, including partial-cleanup reporting if
  credential removal fails. Pairing is complete only after Keychain storage
  succeeds; failed or cancelled setup leaves no active trusted record.
- Implement the local Samsung remote-channel codec and one asynchronous
  WebSocket session for the selected TV over `wss` on port 8002 only. Do not
  open port 8001 or downgrade after a TLS failure. If the target cannot pair
  and reconnect securely on 8002, record that M10 cannot claim live support
  for it and revisit the target or protocol in a later owner decision.
- Add application-owned connect/pair/reconnect operations, timeouts,
  cancellation, selection-generation checks, typed errors, and the sole
  bounded remote-action dispatcher. Keep socket writes serial and nonblocking
  for Iced. A revoked token, certificate change, or host change stops reuse of
  the old credential and requires the appropriate re-pair or trust step.
- Wire the current Remote View actions to that dispatcher. Validate each key
  mapping on the target; disable an unsupported or unverified action with a
  reason. Keep Power Toggle visibly disabled and defer its behavior to the
  wake/power milestone. Keep the exact-volume slider disabled. Defined
  keyboard equivalents must submit the same typed request and respect
  text-field/modal focus and repeat rules.
- Show Connection and Pairing progress, request IDs and terminal outcomes in
  the existing message and Activity regions. Keep sent/written, rejected,
  failed, and uncertain outcomes distinct from observed TV state.
- Build a reproducible locally signed macOS app bundle with a stable bundle
  identifier and `NSLocalNetworkUsageDescription`; add Bonjour service
  declarations if the chosen discovery method requires them. Show permission
  recovery and test the bundle on the target Mac, since a Terminal-launched
  binary may bypass the prompt.

## Out of scope

- Power Toggle, wake or control of a sleeping TV, SmartThings/cloud control,
  legacy H/J protocol support, source changes, app launch, text input, and
  exact volume.
- A claim that socket write acknowledgment proves a key took effect or that a
  lost connection proves the TV turned off.
- Automatic retries of non-idempotent key presses.
- Port 8001 plaintext compatibility, even as an opt-in fallback.
- Broad model support beyond hardware actually tested in this milestone.

## Delivery gates

1. **Hardware and dependency decisions.** On the powered-on target, determine
   reachable endpoints, certificate properties, Pairing behavior, key aliases,
   and available discovery advertisements. Decide which observed device facts
   can bind a saved token to the same TV. Confirm secure 8002 Pairing and
   reconnect before claiming this target is supported. Smoke-test local-network
   access from a locally signed test bundle, not just Terminal. Select
   compatible Rust transport, TLS, Keychain, persistence, and discovery
   dependencies based on those observations. Keep probe credentials and
   addresses outside the repository. If secure connection or bundle permission
   cannot work on the target, record the blocker before building later gates.
2. **Trusted TV setup.** Implement local target validation, discovery/manual
   fallback, identity confirmation, certificate decision, Pairing, Keychain
   storage, saved-TV selection, re-pair, and forget. Exercise each failure with
   fake adapters before using the live TV.
3. **Live session and dispatch.** Implement bounded parsing and encoding,
   generation-scoped session ownership, connection/reconnection policy, and
   the single bounded queue. Connect P1-M9 policy to dispatch admission and
   typed request results.
4. **Presentation and recovery.** Wire Settings and Remote View to application
   commands and events. Render safe, ordered outcomes in both windows. Keep
   controls disabled during trust, Pairing, reconnect, and known unsupported
   conditions. Test keyboard/pointer equivalence and app-wide observer recovery.
5. **Verification.** Run deterministic fake-port/protocol tests, the Cargo
   gates, native macOS bundle/permission review, and the live acceptance script
   below. Record sanitized results and update compatibility documentation.

## Completion checklist

The owner verified empty-install setup and that discovery returns a candidate
through the selected mechanism. The following remaining acceptance work is
deferred without a revisit date: empty/denied/timed-out discovery fallback to
manual address entry; local-network permission alert, denial recovery, and
retry from the signed bundle; re-pair, forget, failed Connection, and switching
TVs during pending work; the remaining native-bundle review; and latency
measurement. Saved-token reconnect and Retry Connection were reported
successful, but the combined recovery item below also requires the deferred
re-pair and forget checks. Enter and Escape keyboard shortcuts work, and
Activity reported all ten key requests as written. The under-150-ms latency
target remains unverified.

- [x] From an empty install, the user can find or manually enter the powered-on
  TV, confirm it, complete Pairing, and see a live Connection.
- [ ] Secure trust is device-scoped; a changed certificate, revoked token, or
  changed host cannot silently reuse a credential. The app never connects to
  port 8001 or downgrades to plaintext.
- [x] Discovery reports a candidate when the tested TV advertises through the
  selected mechanism. The candidate remains untrusted until TV Identity
  Confirmation and Pairing.
- [ ] Empty, denied, or timed-out discovery remains actionable through manual
  address entry without claiming a TV was found.
- [ ] The selected saved TV reconnects after app restart without another
  prompt when the TV still accepts its token. Re-pair, forget, and failed
  Connection paths provide clear recovery.
- [ ] Every enabled Remote View button goes through one current-generation
  policy and bounded queue to one mapped key write; rejection, busy, and
  terminal outcomes are visible and correctly attributed. Power Toggle and
  exact-volume controls remain disabled with accurate reasons.
- [ ] The app bundle has the required local-network privacy metadata and a
  stable test identity. Permission denial, initial alert timing, and retry
  after approval are verified from the bundle, not inferred from `cargo run`.
- [ ] Switching or forgetting TVs cancels stale work and prevents a late
  response or queued command from reaching or changing the new selection.
- [ ] Tokens stay in Keychain and never appear in messages, logs, fixtures,
  ordinary preferences, or committed artifacts. Real LAN addresses, MACs,
  serials, and raw pairing frames never appear in logs, fixtures, or commits;
  saved non-secret connection details stay in local preferences.
- [x] Fake tests cover consent, denial, timeout, malformed and oversized
  events, certificate/host change, token revocation, Keychain failure and
  partial forget, queue full, ordering, cancellation, partial write
  uncertainty, and observer reattachment.
- [x] The live acceptance run verifies at least Up/Down/Left/Right, Select,
  Back, Home, Mute, Volume Up, and Volume Down on the target. Power Toggle is
  deferred to the wake/power milestone.
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
  and `cargo test` pass (91 tests).
- [ ] Native app review records the macOS local-network prompt/recovery and a
  sanitized hardware matrix. The matrix is recorded; prompt/recovery review is
  deferred without a revisit date. Only then mark M10 Done.

## Live acceptance script

Use the bundled app on the target Mac and the already-on KU75UA8090FXKR on the
owner's LAN. The owner can approve the TV's pairing prompt and observe button
responses during this run. Start with isolated test app data and no saved
record; avoid deleting the owner's existing Keychain items or TV approvals.
Pair through the app while observing the TV prompt. Send each listed key once
and compare the visible TV response to the app's reported transport outcome.
Restart and verify token reconnect. Disconnect and reconnect the network,
deny TV approval, revoke the TV's trusted client, and
switch or forget the selection during pending work. Do not send Power Toggle.
Record model/firmware, wired or Wi-Fi, selected port, consent behavior, key
results, elapsed ordinary-key response, and recovery outcome without recording
addresses, identifiers, or credentials. Measure the existing under-150-ms
product target from Mac control activation to visible TV reaction over
repeated ordinary-key presses; record the measurement method and distribution.
Do not infer this latency or TV response from a successful socket write.

## References

- [Architecture and sequencing](milestone-10-architecture.md)
- [Local protocol research](../research/samsung-tv-remote-protocol.md)
- [Security and target-TV questions](../research/samsung-tv-protocol-security.md)
- [Control and monitoring review](../control-monitoring-review.md)
- [Sanitized hardware matrix](milestone-10-hardware-matrix.md)
- [Contribution guide](../contribution-guide.md)
- [Apple local-network privacy guidance](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy)
